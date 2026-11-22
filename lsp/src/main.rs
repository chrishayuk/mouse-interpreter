//! A language server for Mouse.
//!
//! It speaks the Language Server Protocol over standard input and output, so
//! any editor with an LSP client can use it. Everything it knows about a
//! program comes from `src/analysis.rs`, the same code behind `mouse --check`.
//!
//! The server is single-threaded and reads one message at a time, which lets
//! it also be compiled for WASI and run inside an editor with nothing to
//! install (see `editors/vscode`).

#[path = "../../src/analysis.rs"]
mod analysis;

use analysis::{analyse, Analysis, Kind, Severity, Token};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

const SEMANTIC_TYPES: &[&str] = &["comment", "string", "number", "variable", "function", "parameter", "operator", "keyword"];
const SEMANTIC_MODIFIERS: &[&str] = &["definition"];

struct Doc {
    text: String,
    analysis: Analysis,
    line_starts: Vec<usize>,
}

impl Doc {
    fn new(text: String) -> Self {
        let analysis = analyse(&text);
        let mut line_starts = vec![0];
        line_starts.extend(text.bytes().enumerate().filter(|&(_, b)| b == b'\n').map(|(i, _)| i + 1));
        Doc { text, analysis, line_starts }
    }

    fn line_of(&self, offset: usize) -> usize {
        self.line_starts.partition_point(|&start| start <= offset) - 1
    }

    /// A byte offset as an LSP position: a line and a count of UTF-16 units.
    fn position(&self, offset: usize) -> Value {
        let offset = offset.min(self.text.len());
        let line = self.line_of(offset);
        let character: usize = self.text[self.line_starts[line]..offset].chars().map(char::len_utf16).sum();
        json!({ "line": line, "character": character })
    }

    fn range(&self, start: usize, end: usize) -> Value {
        json!({ "start": self.position(start), "end": self.position(end) })
    }

    fn token_range(&self, t: &Token) -> Value {
        self.range(t.start, t.end)
    }

    /// The range of just the letter in `#A`, `$A` or `%A`.
    fn letter_range(&self, t: &Token) -> Value {
        self.range(t.end - 1, t.end)
    }

    fn offset(&self, position: &Value) -> usize {
        let line = position["line"].as_u64().unwrap_or(0) as usize;
        let mut units = position["character"].as_u64().unwrap_or(0) as usize;
        let Some(&start) = self.line_starts.get(line) else {
            return self.text.len();
        };
        let end = self.line_starts.get(line + 1).map_or(self.text.len(), |&next| next - 1);
        let mut offset = start;
        for c in self.text[start..end].chars() {
            if units < c.len_utf16() {
                break;
            }
            units -= c.len_utf16();
            offset += c.len_utf8();
        }
        offset
    }

    fn line_text(&self, offset: usize) -> &str {
        let line = self.line_of(offset);
        let end = self.line_starts.get(line + 1).map_or(self.text.len(), |&next| next - 1);
        self.text[self.line_starts[line]..end].trim_end_matches('\r')
    }

    fn token_at(&self, position: &Value) -> Option<(usize, &Token)> {
        let i = self.analysis.token_at(self.offset(position))?;
        Some((i, &self.analysis.tokens[i]))
    }

    fn region_name(&self, region: usize) -> Option<char> {
        self.analysis.regions[region].name.map(char::from)
    }
}

fn ordinal(n: usize) -> String {
    match n {
        1 => "first".into(),
        2 => "second".into(),
        3 => "third".into(),
        4 => "fourth".into(),
        5 => "fifth".into(),
        n => format!("{n}th"),
    }
}

fn operator_help(ch: u8) -> &'static str {
    match ch {
        b'?' => "**`?` read**\n\nReads a number from the input and pushes it on the stack.",
        b'!' => "**`!` print**\n\nPops the number on top of the stack and prints it.",
        b'+' => "**`+` add**\n\nPops two numbers and pushes their sum. `3 4 +` leaves 7.",
        b'-' => "**`-` subtract**\n\nPops two numbers and pushes the first minus the second. `7 3 -` leaves 4.",
        b'*' => "**`*` multiply**\n\nPops two numbers and pushes their product. `3 4 *` leaves 12.",
        b'/' => "**`/` divide**\n\nPops two numbers and pushes the first divided by the second, as a whole number. `7 2 /` leaves 3.",
        b'.' => "**`.` fetch**\n\nReplaces the address on top of the stack with the value stored there. `X.` is the value of X.",
        b'=' => "**`=` store**\n\nPops a value and an address and stores the value there. `X 5 =` sets X to 5.",
        b'[' => "**`[` if**\n\nPops a number. If it is zero or less, skips to the matching `]`. There is no else.",
        b']' => "**`]` end of an if**\n\nDoes nothing itself: it marks where `[` skips to.",
        b'(' => "**`(` loop**\n\nStarts a loop, which repeats until a `^` inside it leaves.",
        b')' => "**`)` end of a loop**\n\nGoes back to the matching `(`.",
        b'^' => "**`^` leave the loop**\n\nPops a number. If it is zero or less, leaves the loop it is in and carries on after the `)`.",
        b'@' => "**`@` return**\n\nEnds the macro and returns to just after the `;` of the call.",
        b',' => "**`,` argument separator**\n\nStarts the next argument of a macro call.",
        b';' => "**`;` end of a call**\n\nEnds the arguments of a macro call.",
        _ => "",
    }
}

/// A one-line label for a macro call, such as `#M,%A,%B;`.
fn signature(letter: u8, arity: usize) -> String {
    let mut label = format!("#{}", letter as char);
    for i in 0..arity {
        label.push_str(&format!(",%{}", (b'A' + i as u8) as char));
    }
    label.push(';');
    label
}

struct Server {
    docs: HashMap<String, Doc>,
    shutdown: bool,
    exit: bool,
}

impl Server {
    fn new() -> Self {
        Server { docs: HashMap::new(), shutdown: false, exit: false }
    }

    /// Handles one incoming message and returns the messages to send back.
    fn handle(&mut self, message: &Value) -> Vec<Value> {
        let method = message["method"].as_str().unwrap_or("");
        let params = &message["params"];
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();

        let Some(id) = message.get("id").filter(|_| !method.is_empty()) else {
            // A notification, or a reply to something we never ask.
            return match method {
                "textDocument/didOpen" => self.open(uri, params["textDocument"]["text"].as_str().unwrap_or("")),
                "textDocument/didChange" => {
                    // Full sync: the last change holds the whole text.
                    match params["contentChanges"].as_array().and_then(|c| c.last()).and_then(|c| c["text"].as_str()) {
                        Some(text) => self.open(uri, text),
                        None => vec![],
                    }
                }
                "textDocument/didClose" => {
                    self.docs.remove(&uri);
                    vec![json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                        "params": { "uri": uri, "diagnostics": [] } })]
                }
                "exit" => {
                    self.exit = true;
                    vec![]
                }
                _ => vec![],
            };
        };

        let reply = |result: Value| vec![json!({ "jsonrpc": "2.0", "id": id, "result": result })];
        let fail = |code: i32, text: &str| vec![json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": text } })];

        match method {
            "initialize" => return reply(self.capabilities()),
            "shutdown" => {
                self.shutdown = true;
                return reply(Value::Null);
            }
            _ => {}
        }
        if !method.starts_with("textDocument/") {
            return fail(-32601, &format!("method not supported: {method}"));
        }
        let Some(doc) = self.docs.get(&uri) else {
            return reply(Value::Null);
        };
        let position = &params["position"];

        match method {
            "textDocument/hover" => reply(hover(doc, position)),
            "textDocument/definition" => reply(definition(doc, &uri, position)),
            "textDocument/references" => {
                let with_declaration = params["context"]["includeDeclaration"].as_bool().unwrap_or(true);
                reply(json!(uses(doc, position, with_declaration)
                    .iter().map(|r| json!({ "uri": uri, "range": r })).collect::<Vec<_>>()))
            }
            "textDocument/documentHighlight" => reply(json!(uses(doc, position, true)
                .iter().map(|r| json!({ "range": r })).collect::<Vec<_>>())),
            "textDocument/prepareRename" => reply(prepare_rename(doc, position)),
            "textDocument/rename" => match rename(doc, &uri, position, params["newName"].as_str().unwrap_or("")) {
                Ok(edit) => reply(edit),
                Err(text) => fail(-32602, &text),
            },
            "textDocument/completion" => reply(completion(doc, position)),
            "textDocument/signatureHelp" => reply(signature_help(doc, position)),
            "textDocument/documentSymbol" => reply(symbols(doc)),
            "textDocument/foldingRange" => reply(folding(doc)),
            "textDocument/semanticTokens/full" => reply(json!({ "data": semantic_tokens(doc) })),
            _ => fail(-32601, &format!("method not supported: {method}")),
        }
    }

    fn capabilities(&self) -> Value {
        json!({
            "serverInfo": { "name": "mouse-lsp", "version": env!("CARGO_PKG_VERSION") },
            "capabilities": {
                "positionEncoding": "utf-16",
                "textDocumentSync": { "openClose": true, "change": 1 },
                "hoverProvider": true,
                "definitionProvider": true,
                "referencesProvider": true,
                "documentHighlightProvider": true,
                "renameProvider": { "prepareProvider": true },
                "completionProvider": { "triggerCharacters": ["#", "%"] },
                "signatureHelpProvider": { "triggerCharacters": [","], "retriggerCharacters": [","] },
                "documentSymbolProvider": true,
                "foldingRangeProvider": true,
                "semanticTokensProvider": {
                    "legend": { "tokenTypes": SEMANTIC_TYPES, "tokenModifiers": SEMANTIC_MODIFIERS },
                    "full": true,
                },
            },
        })
    }

    fn open(&mut self, uri: String, text: &str) -> Vec<Value> {
        let doc = Doc::new(text.to_string());
        let diagnostics: Vec<Value> = doc.analysis.diagnostics.iter().map(|d| {
            let mut out = json!({
                "range": doc.range(d.start, d.end),
                "severity": match d.severity { Severity::Error => 1, Severity::Warning => 2, Severity::Hint => 4 },
                "code": d.code,
                "source": "mouse",
                "message": d.message,
            });
            if d.unnecessary {
                out["tags"] = json!([1]);
            }
            out
        }).collect();
        self.docs.insert(uri.clone(), doc);
        vec![json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
            "params": { "uri": uri, "diagnostics": diagnostics } })]
    }
}

fn hover(doc: &Doc, position: &Value) -> Value {
    let Some((_, t)) = doc.token_at(position) else {
        return Value::Null;
    };
    let a = &doc.analysis;
    let letter = t.ch as char;
    let call_name = |i: Option<usize>| i.map(|i| a.tokens[i].ch as char);
    let text = match t.kind {
        Kind::Comment => return Value::Null,
        Kind::Str => "**String**\n\nPrints its text. A `!` inside a string prints a new line.".to_string(),
        Kind::Number => "**Number**\n\nPushes the number on the stack.".to_string(),
        Kind::Variable => {
            let place = match doc.region_name(t.region) {
                Some(name) => format!("It is local to this call of `${name}`: every macro call gets 26 variables of its own."),
                None => format!("In the main program its address is {}.", t.ch - b'A' + 1),
            };
            format!("**Variable `{letter}`**\n\nPushes the address of `{letter}` on the stack. `{letter}.` fetches its value, and `{letter} 5 =` stores 5 in it.\n\n{place}")
        }
        Kind::Def | Kind::Call => {
            let mut text = match (t.kind, a.definition(t.ch)) {
                (Kind::Def, _) => format!("**Macro `${letter}`**"),
                (_, Some(_)) => format!("**Call of macro `${letter}`**"),
                (_, None) => format!("**Call of macro `${letter}`**, which is not defined. The call does nothing."),
            };
            if let Some(def) = a.definition(t.ch) {
                let arity = a.arity(t.ch);
                text.push_str(&format!("\n\n```mouse\n{}\n```", doc.line_text(a.tokens[def].start).trim()));
                if let Some(comment) = a.macro_doc(&doc.text, t.ch) {
                    text.push_str(&format!("\n\n{comment}"));
                }
                let calls = a.macro_uses(t.ch).iter().filter(|&&i| a.tokens[i].kind == Kind::Call).count();
                text.push_str(&format!("\n\nUses {} and is called {}.",
                    match arity { 0 => "no parameters".to_string(), 1 => "one parameter, `%A`".to_string(),
                        n => format!("{n} parameters, `%A` to `%{}`", (b'A' + n as u8 - 1) as char) },
                    match calls { 0 => "nowhere".to_string(), 1 => "once".to_string(), n => format!("{n} times") }));
            }
            text
        }
        Kind::Param => {
            let n = (t.ch - b'A') as usize + 1;
            let of = doc.region_name(t.region).map_or(String::new(), |name| format!(" of the call to `${name}`"));
            format!("**Parameter `%{letter}`**\n\nThe {} argument{of}. The argument is run each time `%{letter}` is reached, with the caller's variables, so it can be a value, a calculation or a whole piece of program.", ordinal(n))
        }
        Kind::End => "**`$$` end of the program**\n\nNothing after it is part of the program.".to_string(),
        Kind::Dollar => "**`$`**\n\nOn its own this ends the program. `$A` starts a macro definition and `$$` ends the program.".to_string(),
        Kind::Trailer => "Text after `$$` is not part of the program.".to_string(),
        Kind::BadCall => "**`#` macro call**\n\nMust be followed by the letter of a macro: `#A,1,2;`.".to_string(),
        Kind::BadParam => "**`%` parameter**\n\nMust be followed by a letter: `%A` is the first argument of the call.".to_string(),
        Kind::Unknown => "This text is not a Mouse instruction. The interpreter ignores it.".to_string(),
        Kind::Op => {
            let mut text = operator_help(t.ch).to_string();
            match (t.ch, call_name(t.partner)) {
                (b',', Some(name)) => text.push_str(&format!("\n\nHere it starts the {} argument of `#{name}`.", ordinal(t.arg))),
                (b';', Some(name)) => text.push_str(&format!("\n\nHere it ends the call `#{name}`.")),
                _ => {}
            }
            text
        }
    };
    json!({ "contents": { "kind": "markdown", "value": text }, "range": doc.token_range(t) })
}

fn definition(doc: &Doc, uri: &str, position: &Value) -> Value {
    match doc.token_at(position) {
        Some((_, t)) if matches!(t.kind, Kind::Call | Kind::Def) => match doc.analysis.definition(t.ch) {
            Some(def) => json!({ "uri": uri, "range": doc.token_range(&doc.analysis.tokens[def]) }),
            None => Value::Null,
        },
        _ => Value::Null,
    }
}

/// The tokens that mean the same thing as the one under the cursor, and the
/// part of each that is its name.
fn related(doc: &Doc, position: &Value) -> Vec<usize> {
    let a = &doc.analysis;
    match doc.token_at(position) {
        Some((_, t)) if matches!(t.kind, Kind::Call | Kind::Def) => a.macro_uses(t.ch),
        Some((_, t)) if t.kind == Kind::Variable => a.variable_uses(t.ch, t.region),
        Some((_, t)) if t.kind == Kind::Param => (0..a.tokens.len())
            .filter(|&i| a.tokens[i].kind == Kind::Param && a.tokens[i].ch == t.ch && a.tokens[i].region == t.region)
            .collect(),
        Some((i, t)) if t.kind == Kind::Op && b"[]()".contains(&t.ch) => {
            let mut pair = vec![i];
            pair.extend(t.partner);
            pair.sort();
            pair
        }
        _ => vec![],
    }
}

fn uses(doc: &Doc, position: &Value, with_declaration: bool) -> Vec<Value> {
    related(doc, position)
        .into_iter()
        .map(|i| &doc.analysis.tokens[i])
        .filter(|t| with_declaration || t.kind != Kind::Def)
        .map(|t| match t.kind {
            Kind::Variable | Kind::Op => doc.token_range(t),
            _ => doc.letter_range(t),
        })
        .collect()
}

fn prepare_rename(doc: &Doc, position: &Value) -> Value {
    match doc.token_at(position) {
        Some((_, t)) if matches!(t.kind, Kind::Call | Kind::Def | Kind::Variable) => {
            json!({ "range": doc.letter_range(t), "placeholder": (t.ch as char).to_string() })
        }
        _ => Value::Null,
    }
}

fn rename(doc: &Doc, uri: &str, position: &Value, new_name: &str) -> Result<Value, String> {
    let a = &doc.analysis;
    let Some((_, t)) = doc.token_at(position) else {
        return Err("there is nothing to rename here".into());
    };
    let new = match new_name.as_bytes() {
        [letter] if letter.is_ascii_uppercase() => *letter,
        _ => return Err("a Mouse name is a single capital letter, A to Z".into()),
    };
    let targets = match t.kind {
        Kind::Call | Kind::Def => {
            if new != t.ch && a.definition(new).is_some() {
                return Err(format!("there is already a macro ${}", new as char));
            }
            a.macro_uses(t.ch)
        }
        Kind::Variable => {
            if new != t.ch && !a.variable_uses(new, t.region).is_empty() {
                let place = doc.region_name(t.region).map_or("the main program".to_string(), |name| format!("macro ${name}"));
                return Err(format!("{place} already uses the variable {}", new as char));
            }
            a.variable_uses(t.ch, t.region)
        }
        _ => return Err("only variables and macros can be renamed".into()),
    };
    let edits: Vec<Value> = targets
        .into_iter()
        .map(|i| json!({ "range": doc.letter_range(&a.tokens[i]), "newText": new_name }))
        .collect();
    Ok(json!({ "changes": { uri: edits } }))
}

fn completion(doc: &Doc, position: &Value) -> Value {
    let a = &doc.analysis;
    let offset = doc.offset(position);
    let before = doc.text.as_bytes()[..offset].iter().rev().copied().find(|b| !matches!(b, b' ' | b'\t'));
    let mut items = Vec::new();

    let macros: Vec<u8> = (b'A'..=b'Z').filter(|&letter| a.definition(letter).is_some()).collect();
    match before {
        Some(b'#') => {
            for letter in macros {
                let def = a.definition(letter).unwrap();
                items.push(json!({
                    "label": (letter as char).to_string(),
                    "kind": 3,
                    "detail": doc.line_text(a.tokens[def].start).trim(),
                    "documentation": a.macro_doc(&doc.text, letter).unwrap_or_default(),
                }));
            }
        }
        Some(b'%') => {
            // Offer the parameters the macro already uses, and one more.
            let region = a.regions.iter().position(|r| r.start <= offset && offset <= r.end && r.name.is_some());
            let used = region.and_then(|r| a.regions[r].name).map_or(0, |name| a.arity(name));
            for i in 0..(used + 1).min(26) {
                items.push(json!({
                    "label": ((b'A' + i as u8) as char).to_string(),
                    "kind": 6,
                    "detail": format!("the {} argument of the call", ordinal(i + 1)),
                }));
            }
        }
        _ => {
            for letter in macros {
                let arity = a.arity(letter);
                let arguments: String = (1..=arity).map(|i| format!(",${{{i}}}")).collect();
                items.push(json!({
                    "label": signature(letter, arity),
                    "kind": 3,
                    "detail": a.macro_doc(&doc.text, letter).unwrap_or_else(|| format!("call macro ${}", letter as char)),
                    "insertText": format!("#{}{arguments};", letter as char),
                    "insertTextFormat": 2,
                }));
            }
            for (label, detail, insert) in [
                ("loop", "( … ^ … ) repeat while a number is above zero", "( ${1:N.} ^\n  $0\n)"),
                ("if", "[ … ] run when the number is above zero", "[ $0 ]"),
                ("macro", "$A … @ define a macro", "\\$${1:A} $0 @"),
                ("print", "\"…!\" print text and a new line", "\"$0!\""),
                ("end", "$$ end of the program", "\\$\\$"),
            ] {
                items.push(json!({ "label": label, "kind": 15, "detail": detail, "insertText": insert, "insertTextFormat": 2 }));
            }
        }
    }
    json!(items)
}

fn signature_help(doc: &Doc, position: &Value) -> Value {
    let a = &doc.analysis;
    let offset = doc.offset(position);
    // The calls still open at the cursor, innermost last, with commas seen.
    let mut open: Vec<(usize, usize)> = Vec::new();
    for (i, t) in a.tokens.iter().enumerate().take_while(|(_, t)| t.end <= offset) {
        match t.kind {
            Kind::Call => open.push((i, 0)),
            Kind::Op if t.ch == b',' && t.partner.is_some() => {
                if let Some(top) = open.last_mut() {
                    top.1 += 1;
                }
            }
            Kind::Op if t.ch == b';' && t.partner.is_some() => {
                open.pop();
            }
            Kind::Def | Kind::End => open.clear(),
            _ => {}
        }
    }
    let Some(&(call, commas)) = open.last() else {
        return Value::Null;
    };
    let letter = a.tokens[call].ch;
    if a.definition(letter).is_none() {
        return Value::Null;
    }
    let arity = a.arity(letter);
    let label = signature(letter, arity);
    let parameters: Vec<Value> = (0..arity)
        .map(|i| json!({ "label": [2 + i * 3 + 1, 2 + i * 3 + 3] }))
        .collect();
    let mut help = json!({ "label": label, "parameters": parameters });
    if let Some(comment) = a.macro_doc(&doc.text, letter) {
        help["documentation"] = json!(comment);
    }
    json!({ "signatures": [help], "activeSignature": 0, "activeParameter": commas.saturating_sub(1) })
}

fn symbols(doc: &Doc) -> Value {
    let a = &doc.analysis;
    let mut out = Vec::new();
    for region in &a.regions {
        let (Some(name), Some(def)) = (region.name, region.def) else {
            continue;
        };
        let end = doc.text[..region.end].trim_end().len().max(a.tokens[def].end);
        out.push(json!({
            "name": format!("${}", name as char),
            "detail": a.macro_doc(&doc.text, name).unwrap_or_default().lines().next().unwrap_or(""),
            "kind": 12,
            "range": doc.range(region.start, end),
            "selectionRange": doc.token_range(&a.tokens[def]),
        }));
    }
    json!(out)
}

fn folding(doc: &Doc) -> Value {
    let a = &doc.analysis;
    let mut out = Vec::new();
    let mut fold = |start: usize, end: usize, kind: Option<&str>| {
        let (first, last) = (doc.line_of(start), doc.line_of(end));
        if last > first {
            let mut range = json!({ "startLine": first, "endLine": last });
            if let Some(kind) = kind {
                range["kind"] = json!(kind);
            }
            out.push(range);
        }
    };
    for region in a.regions.iter().filter(|r| r.name.is_some()) {
        let end = doc.text[..region.end].trim_end().len();
        fold(region.start, end.max(region.start), None);
    }
    for (i, t) in a.tokens.iter().enumerate() {
        if let (Kind::Op, b'[' | b'(', Some(partner)) = (t.kind, t.ch, t.partner) {
            if partner > i {
                fold(t.start, a.tokens[partner].start, None);
            }
        }
    }
    // Runs of comment lines.
    let comments: Vec<&Token> = a.tokens.iter().filter(|t| t.kind == Kind::Comment).collect();
    let mut i = 0;
    while i < comments.len() {
        let mut j = i;
        while j + 1 < comments.len() && doc.line_of(comments[j + 1].start) == doc.line_of(comments[j].start) + 1 {
            j += 1;
        }
        fold(comments[i].start, comments[j].start, Some("comment"));
        i = j + 1;
    }
    json!(out)
}

/// Semantic tokens, in the protocol's five-numbers-per-token encoding.
fn semantic_tokens(doc: &Doc) -> Vec<u32> {
    let type_index = |name: &str| SEMANTIC_TYPES.iter().position(|&t| t == name).unwrap() as u32;
    let mut data = Vec::new();
    let (mut last_line, mut last_start) = (0u32, 0u32);
    for t in &doc.analysis.tokens {
        let (kind, modifiers) = match t.kind {
            Kind::Comment | Kind::Trailer => ("comment", 0),
            Kind::Str => ("string", 0),
            Kind::Number => ("number", 0),
            Kind::Variable => ("variable", 0),
            Kind::Def => ("function", 1),
            Kind::Call => ("function", 0),
            Kind::Param => ("parameter", 0),
            Kind::End | Kind::Dollar => ("keyword", 0),
            Kind::Op if b"[]()^@".contains(&t.ch) => ("keyword", 0),
            Kind::Op | Kind::BadCall | Kind::BadParam => ("operator", 0),
            Kind::Unknown => continue,
        };
        // A token may not span lines, so a multi-line string is sent in pieces.
        let mut start = t.start;
        while start < t.end {
            let line = doc.line_of(start);
            let line_end = doc.line_starts.get(line + 1).map_or(doc.text.len(), |&next| next - 1);
            let end = t.end.min(line_end);
            let text = doc.text[start..end].trim_end_matches('\r');
            let length: usize = text.chars().map(char::len_utf16).sum();
            if length > 0 {
                let column = doc.position(start)["character"].as_u64().unwrap() as u32;
                let line = line as u32;
                let delta_line = line - last_line;
                let delta_start = if delta_line == 0 { column - last_start } else { column };
                data.extend([delta_line, delta_start, length as u32, type_index(kind), modifiers]);
                (last_line, last_start) = (line, column);
            }
            start = line_end + 1;
        }
    }
    data
}

fn read_message(input: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let line = line.trim_end();
        if line.is_empty() {
            if length.is_some() {
                break;
            }
            continue;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse::<usize>().ok();
        }
    }
    let mut body = vec![0; length.unwrap()];
    input.read_exact(&mut body)?;
    serde_json::from_slice(&body).map(Some).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn main() -> io::Result<()> {
    if std::env::args().any(|arg| arg == "--version") {
        println!("mouse-lsp {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut output = io::stdout().lock();
    let mut server = Server::new();
    while let Some(message) = read_message(&mut input)? {
        for reply in server.handle(&message) {
            let body = reply.to_string();
            write!(output, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
        }
        output.flush()?;
        if server.exit {
            break;
        }
    }
    std::process::exit(if server.shutdown { 0 } else { 1 });
}

#[cfg(test)]
mod tests {
    use super::*;

    const URI: &str = "file:///test.mouse";
    const PROGRAM: &str = "\
' Lane's test, with an extra macro.
N 5 =
#A,#B;; #M,7,N.; !

$A Q 1 = %A Q. ! @
$B Q 33 = @
$M %A %B - @   ' M(a, b) = a - b
$$
";

    fn server_with(text: &str) -> (Server, Vec<Value>) {
        let mut server = Server::new();
        let out = server.handle(&json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": { "textDocument": { "uri": URI, "languageId": "mouse", "version": 1, "text": text } } }));
        (server, out)
    }

    fn ask(server: &mut Server, method: &str, line: u32, character: u32, extra: Value) -> Value {
        let mut params = json!({ "textDocument": { "uri": URI }, "position": { "line": line, "character": character } });
        for (key, value) in extra.as_object().into_iter().flatten() {
            params[key] = value.clone();
        }
        let mut out = server.handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }));
        assert_eq!(out.len(), 1);
        out.remove(0)
    }

    #[test]
    fn initialise_advertises_what_is_implemented() {
        let mut server = Server::new();
        let out = server.handle(&json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {} }));
        let capabilities = &out[0]["result"]["capabilities"];
        assert_eq!(capabilities["hoverProvider"], true);
        assert_eq!(capabilities["renameProvider"]["prepareProvider"], true);
        assert_eq!(capabilities["semanticTokensProvider"]["legend"]["tokenTypes"][4], "function");
    }

    #[test]
    fn a_good_program_publishes_no_diagnostics_and_a_bad_one_does() {
        let (_, out) = server_with(PROGRAM);
        assert_eq!(out[0]["method"], "textDocument/publishDiagnostics");
        assert_eq!(out[0]["params"]["diagnostics"], json!([]));

        let (_, out) = server_with("1 [ 2 !\n#Z;\n$$\n");
        let diagnostics = out[0]["params"]["diagnostics"].as_array().unwrap();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0]["severity"], 1);
        assert_eq!(diagnostics[0]["code"], "unclosed-bracket");
        assert_eq!(diagnostics[0]["range"], json!({ "start": { "line": 0, "character": 2 }, "end": { "line": 0, "character": 3 } }));
        assert_eq!(diagnostics[1]["code"], "undefined-macro");
        assert_eq!(diagnostics[1]["severity"], 2);
    }

    #[test]
    fn hover_explains_instructions_macros_and_parameters() {
        let (mut server, _) = server_with(PROGRAM);
        let text = |v: Value| v["result"]["contents"]["value"].as_str().unwrap_or("").to_string();
        assert!(text(ask(&mut server, "textDocument/hover", 1, 4, json!({}))).contains("store"));
        let call = text(ask(&mut server, "textDocument/hover", 2, 9, json!({})));
        assert!(call.contains("$M %A %B - @"), "{call}");
        assert!(call.contains("M(a, b) = a - b"));
        assert!(call.contains("2 parameters") && call.contains("called once"));
        assert!(text(ask(&mut server, "textDocument/hover", 4, 10, json!({}))).contains("first argument of the call to `$A`"));
        assert!(text(ask(&mut server, "textDocument/hover", 4, 3, json!({}))).contains("local to this call of `$A`"));
        assert!(text(ask(&mut server, "textDocument/hover", 1, 0, json!({}))).contains("address is 14"));
        assert!(text(ask(&mut server, "textDocument/hover", 2, 12, json!({}))).contains("second argument of `#M`"));
        assert_eq!(ask(&mut server, "textDocument/hover", 3, 0, json!({}))["result"], Value::Null);
    }

    #[test]
    fn definition_and_references_follow_macros_and_local_variables() {
        let (mut server, _) = server_with(PROGRAM);
        let definition = ask(&mut server, "textDocument/definition", 2, 4, json!({}));
        assert_eq!(definition["result"]["range"]["start"], json!({ "line": 5, "character": 0 }));

        let references = ask(&mut server, "textDocument/references", 5, 1, json!({ "context": { "includeDeclaration": true } }));
        assert_eq!(references["result"].as_array().unwrap().len(), 2);
        let calls = ask(&mut server, "textDocument/references", 5, 1, json!({ "context": { "includeDeclaration": false } }));
        assert_eq!(calls["result"].as_array().unwrap().len(), 1);

        // Q in $A and Q in $B are different variables.
        let q = ask(&mut server, "textDocument/documentHighlight", 4, 3, json!({}));
        let lines: Vec<u64> = q["result"].as_array().unwrap().iter().map(|r| r["range"]["start"]["line"].as_u64().unwrap()).collect();
        assert_eq!(lines, [4, 4]);
    }

    #[test]
    fn rename_changes_a_macro_everywhere_and_refuses_a_clash() {
        let (mut server, _) = server_with(PROGRAM);
        let prepared = ask(&mut server, "textDocument/prepareRename", 2, 1, json!({}));
        assert_eq!(prepared["result"]["placeholder"], "A");
        assert_eq!(prepared["result"]["range"]["start"]["character"], 1);

        let renamed = ask(&mut server, "textDocument/rename", 2, 1, json!({ "newName": "X" }));
        let edits = renamed["result"]["changes"][URI].as_array().unwrap();
        assert_eq!(edits.len(), 2);
        assert!(edits.iter().all(|e| e["newText"] == "X"));

        let clash = ask(&mut server, "textDocument/rename", 2, 1, json!({ "newName": "B" }));
        assert!(clash["error"]["message"].as_str().unwrap().contains("already a macro $B"));
        let bad = ask(&mut server, "textDocument/rename", 2, 1, json!({ "newName": "total" }));
        assert!(bad["error"]["message"].as_str().unwrap().contains("single capital letter"));

        let variable = ask(&mut server, "textDocument/rename", 4, 3, json!({ "newName": "R" }));
        assert_eq!(variable["result"]["changes"][URI].as_array().unwrap().len(), 2);
    }

    #[test]
    fn completion_offers_macros_after_a_hash_and_parameters_after_a_percent() {
        let (mut server, _) = server_with("#\n$A %A % @\n$M %A %B - @\n$$\n");
        let labels = |v: Value| v["result"].as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap().to_string()).collect::<Vec<_>>();
        assert_eq!(labels(ask(&mut server, "textDocument/completion", 0, 1, json!({}))), ["A", "M"]);
        assert_eq!(labels(ask(&mut server, "textDocument/completion", 1, 7, json!({}))), ["A", "B"]);
        let general = labels(ask(&mut server, "textDocument/completion", 3, 0, json!({})));
        assert!(general.contains(&"#M,%A,%B;".to_string()) && general.contains(&"loop".to_string()));
    }

    #[test]
    fn signature_help_tracks_the_argument_under_the_cursor() {
        let (mut server, _) = server_with(PROGRAM);
        let help = ask(&mut server, "textDocument/signatureHelp", 2, 13, json!({}));
        assert_eq!(help["result"]["signatures"][0]["label"], "#M,%A,%B;");
        assert_eq!(help["result"]["signatures"][0]["parameters"][1]["label"], json!([6, 8]));
        assert_eq!(help["result"]["activeParameter"], 1);
        let first = ask(&mut server, "textDocument/signatureHelp", 2, 11, json!({}));
        assert_eq!(first["result"]["activeParameter"], 0);
        assert_eq!(ask(&mut server, "textDocument/signatureHelp", 1, 2, json!({}))["result"], Value::Null);
    }

    #[test]
    fn symbols_folding_and_semantic_tokens() {
        let (mut server, _) = server_with("' one\n' two\n( N. ^\n  N. !\n)\n#M,1;\n$M \"a\nb\" %A\n@\n$$\n");
        let symbols = ask(&mut server, "textDocument/documentSymbol", 0, 0, json!({}));
        assert_eq!(symbols["result"][0]["name"], "$M");
        assert_eq!(symbols["result"][0]["range"]["end"]["line"], 8);

        let folds = ask(&mut server, "textDocument/foldingRange", 0, 0, json!({}));
        let folds: Vec<(u64, u64)> = folds["result"].as_array().unwrap().iter()
            .map(|f| (f["startLine"].as_u64().unwrap(), f["endLine"].as_u64().unwrap())).collect();
        assert!(folds.contains(&(0, 1)) && folds.contains(&(2, 4)) && folds.contains(&(6, 8)), "{folds:?}");

        let tokens = ask(&mut server, "textDocument/semanticTokens/full", 0, 0, json!({}));
        let data: Vec<u64> = tokens["result"]["data"].as_array().unwrap().iter().map(|n| n.as_u64().unwrap()).collect();
        assert_eq!(data.len() % 5, 0);
        assert_eq!(&data[..5], [0, 0, 5, 0, 0]);
        // The two-line string is sent as two tokens of type "string".
        let strings = data.chunks(5).filter(|c| c[3] == 1).count();
        assert_eq!(strings, 2);
        // "$M" is a function with the definition modifier.
        assert!(data.chunks(5).any(|c| c[3] == 4 && c[4] == 1 && c[2] == 2));
    }

    #[test]
    fn positions_count_utf16_units() {
        let (mut server, out) = server_with("\"😀 é\" x\n$$\n");
        let range = &out[0]["params"]["diagnostics"][0]["range"];
        assert_eq!(range["start"]["character"], 7);
        let hover = ask(&mut server, "textDocument/hover", 0, 7, json!({}));
        assert!(hover["result"]["contents"]["value"].as_str().unwrap().contains("not a Mouse instruction"));
    }

    #[test]
    fn messages_are_framed_with_a_content_length() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"shutdown"}"#;
        let framed = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        let mut input = io::BufReader::new(framed.as_bytes());
        let message = read_message(&mut input).unwrap().unwrap();
        assert_eq!(message["method"], "shutdown");
        assert!(read_message(&mut input).unwrap().is_none());
    }
}
