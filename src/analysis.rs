//! Static analysis of Mouse source text.
//!
//! The interpreter only finds a mistake when execution reaches it. This
//! module reads a program without running it: it splits the text into tokens,
//! pairs up brackets, loops and macro calls, and reports what is certainly or
//! probably wrong. `mouse --check` prints the result, and the language server
//! in `lsp/` builds its editor features on the same tokens.
//!
//! All offsets are byte offsets into the source. Every character that means
//! something in Mouse is ASCII, so token boundaries always fall on character
//! boundaries even when strings and comments contain other text.

// The interpreter binary uses only part of this; the language server the rest.
#![allow(dead_code)]

use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// `'` to the end of the line.
    Comment,
    /// `"..."`, which prints its text.
    Str,
    Number,
    /// A letter A-Z: the address of a variable.
    Variable,
    /// `$A`: the start of a macro definition.
    Def,
    /// `$$`: the end of the program.
    End,
    /// A `$` that is neither of the above.
    Dollar,
    /// `#A`: a macro call. Its arguments follow, up to the matching `;`.
    Call,
    /// `#` with no letter after it.
    BadCall,
    /// `%A`: a formal parameter.
    Param,
    /// `%` with no letter after it.
    BadParam,
    /// One of `? ! + - * / . = [ ] ( ) ^ @ , ;`.
    Op,
    /// Text the interpreter ignores.
    Unknown,
    /// Everything after `$$`.
    Trailer,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
    /// The letter of a variable, definition, call or parameter; the character
    /// of an operator; otherwise 0.
    pub ch: u8,
    /// Which region (main program or macro body) the token is in.
    pub region: usize,
    /// The other end of a pair: `[` and `]`, `(` and `)`, a call and its `;`.
    /// For `^` it is the loop's `(`, and for `,` the call it belongs to.
    pub partner: Option<usize>,
    /// The innermost macro call whose arguments contain this token.
    pub call: Option<usize>,
    /// Which argument of that call: 1 for the first, 0 before the first comma.
    pub arg: usize,
    /// False for a string with no closing quote.
    pub terminated: bool,
}

impl Token {
    pub fn is_op(&self, ch: u8) -> bool {
        self.kind == Kind::Op && self.ch == ch
    }
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

/// The main program, or the body of one macro definition.
#[derive(Clone, Debug)]
pub struct Region {
    /// The macro's letter; `None` for the main program.
    pub name: Option<u8>,
    /// The `$A` token that starts the region.
    pub def: Option<usize>,
    pub start: usize,
    pub end: usize,
    pub tokens: Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Hint,
}

impl Severity {
    pub fn name(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Hint => "hint",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub start: usize,
    pub end: usize,
    pub severity: Severity,
    /// A short stable name for the kind of problem.
    pub code: &'static str,
    pub message: String,
    /// The text is unused, and an editor may fade it.
    pub unnecessary: bool,
}

pub struct Analysis {
    pub tokens: Vec<Token>,
    pub regions: Vec<Region>,
    pub diagnostics: Vec<Diagnostic>,
    /// For each letter, the definition the interpreter uses: the last one.
    pub definitions: [Option<usize>; 26],
}

const OPERATORS: &[u8] = b"?!+-*/.=[]()^@,;";

fn is_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r')
}

fn token(kind: Kind, start: usize, end: usize, ch: u8) -> Token {
    Token {
        kind,
        start,
        end,
        ch,
        region: 0,
        partner: None,
        call: None,
        arg: 0,
        terminated: true,
    }
}

/// Splits the source into tokens. Whitespace is dropped.
fn lex(src: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < src.len() {
        let c = src[i];
        let start = i;
        if is_space(c) {
            i += 1;
        } else if c == b'\'' {
            while i < src.len() && src[i] != b'\n' {
                i += 1;
            }
            let end = if i > start && src[i - 1] == b'\r' { i - 1 } else { i };
            tokens.push(token(Kind::Comment, start, end, 0));
        } else if c == b'"' {
            i += 1;
            while i < src.len() && src[i] != b'"' {
                i += 1;
            }
            let terminated = i < src.len();
            if terminated {
                i += 1;
            }
            let mut t = token(Kind::Str, start, i, 0);
            t.terminated = terminated;
            tokens.push(t);
        } else if c.is_ascii_digit() {
            while i < src.len() && src[i].is_ascii_digit() {
                i += 1;
            }
            tokens.push(token(Kind::Number, start, i, 0));
        } else if c == b'$' {
            match src.get(i + 1) {
                Some(b'$') => {
                    tokens.push(token(Kind::End, start, i + 2, 0));
                    let mut rest = i + 2;
                    while rest < src.len() && is_space(src[rest]) {
                        rest += 1;
                    }
                    if rest < src.len() {
                        tokens.push(token(Kind::Trailer, rest, src.len(), 0));
                    }
                    break;
                }
                Some(&next) if next.is_ascii_uppercase() => {
                    tokens.push(token(Kind::Def, start, i + 2, next));
                    i += 2;
                }
                _ => {
                    tokens.push(token(Kind::Dollar, start, i + 1, 0));
                    i += 1;
                }
            }
        } else if c == b'#' || c == b'%' {
            // The loader drops whitespace, so "# A" is the same call as "#A".
            let mut j = i + 1;
            while j < src.len() && is_space(src[j]) {
                j += 1;
            }
            let (good, bad) = if c == b'#' {
                (Kind::Call, Kind::BadCall)
            } else {
                (Kind::Param, Kind::BadParam)
            };
            match src.get(j) {
                Some(&letter) if letter.is_ascii_uppercase() => {
                    tokens.push(token(good, start, j + 1, letter));
                    i = j + 1;
                }
                _ => {
                    tokens.push(token(bad, start, i + 1, 0));
                    i += 1;
                }
            }
        } else if c.is_ascii_uppercase() {
            tokens.push(token(Kind::Variable, start, i + 1, c));
            i += 1;
        } else if OPERATORS.contains(&c) {
            tokens.push(token(Kind::Op, start, i + 1, c));
            i += 1;
        } else {
            // A run of text with no meaning, such as a lower-case word.
            let significant = |b: u8| {
                is_space(b)
                    || b.is_ascii_uppercase()
                    || b.is_ascii_digit()
                    || OPERATORS.contains(&b)
                    || matches!(b, b'\'' | b'"' | b'$' | b'#' | b'%')
            };
            let word = c.is_ascii_lowercase();
            i += 1;
            while i < src.len() && !significant(src[i]) && (src[i].is_ascii_lowercase() == word || src[i] >= 0x80) {
                i += 1;
            }
            tokens.push(token(Kind::Unknown, start, i, 0));
        }
    }
    tokens
}

struct Checker<'a> {
    src: &'a [u8],
    diagnostics: Vec<Diagnostic>,
}

impl Checker<'_> {
    fn report(&mut self, t: &Token, severity: Severity, code: &'static str, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic {
            start: t.start,
            end: t.end,
            severity,
            code,
            message: message.into(),
            unnecessary: false,
        });
    }
    fn text(&self, t: &Token) -> String {
        String::from_utf8_lossy(&self.src[t.start..t.end]).into_owned()
    }
}

/// Analyses a whole program.
pub fn analyse(source: &str) -> Analysis {
    let src = source.as_bytes();
    let mut tokens = lex(src);
    let mut check = Checker { src, diagnostics: Vec::new() };
    let mut regions = vec![Region { name: None, def: None, start: 0, end: src.len(), tokens: 0..tokens.len() }];

    // Open '[', '(' and macro calls, innermost last.
    let mut open: Vec<usize> = Vec::new();
    // For each call token, how many commas have been seen.
    let mut commas = vec![0usize; tokens.len()];
    // For each '(' token, whether something inside can leave the loop.
    let mut has_exit = vec![false; tokens.len()];
    let mut ended = false;

    fn close_region(check: &mut Checker, tokens: &[Token], open: &mut Vec<usize>, region: &mut Region, end: usize, upto: usize) {
        for &i in open.iter() {
            let t = &tokens[i];
            match (t.kind, t.ch) {
                (Kind::Call, letter) => check.report(t, Severity::Error, "unclosed-call",
                    format!("the call #{} has no closing ';'", letter as char)),
                (_, b'[') => check.report(t, Severity::Error, "unclosed-bracket", "'[' is never closed by ']'"),
                _ => check.report(t, Severity::Error, "unclosed-loop", "'(' is never closed by ')'"),
            }
        }
        open.clear();
        region.end = end;
        region.tokens.end = upto;
        if let (Some(letter), Some(def)) = (region.name, region.def) {
            let last = tokens[region.tokens.clone()].iter().rev().find(|t| t.kind != Kind::Comment);
            if !last.is_some_and(|t| t.is_op(b'@')) {
                check.report(&tokens[def], Severity::Warning, "no-return",
                    format!("macro ${} does not end with '@': running off its end stops the whole program", letter as char));
            }
        }
    }

    for i in 0..tokens.len() {
        let kind = tokens[i].kind;
        let ch = tokens[i].ch;
        if kind == Kind::Comment {
            tokens[i].region = regions.len() - 1;
            continue;
        }
        if kind == Kind::Def || kind == Kind::End {
            let start = tokens[i].start;
            let mut region = regions.pop().unwrap();
            close_region(&mut check, &tokens, &mut open, &mut region, start, i);
            regions.push(region);
            if kind == Kind::Def {
                regions.push(Region { name: Some(ch), def: Some(i), start, end: src.len(), tokens: i + 1..tokens.len() });
                tokens[i].region = regions.len() - 1;
            } else {
                ended = true;
                // Nothing after "$$" is part of the program.
                regions.push(Region { name: None, def: None, start, end: src.len(), tokens: i..tokens.len() });
                for t in &mut tokens[i..] {
                    t.region = regions.len() - 1;
                }
                break;
            }
            continue;
        }

        let region = regions.len() - 1;
        let in_macro = regions[region].name.is_some();
        let call = open.iter().rev().copied().find(|&j| tokens[j].kind == Kind::Call);
        tokens[i].region = region;
        tokens[i].call = call;
        tokens[i].arg = call.map_or(0, |j| commas[j]);
        let top = open.last().copied();
        let top_is = |tokens: &[Token], c: u8| top.is_some_and(|j| tokens[j].kind == Kind::Op && tokens[j].ch == c);
        let top_is_call = top.is_some_and(|j| tokens[j].kind == Kind::Call);
        let t = tokens[i].clone();

        match (kind, ch) {
            (Kind::Op, b'[') | (Kind::Op, b'(') | (Kind::Call, _) => open.push(i),
            (Kind::Op, b']') => {
                if top_is(&tokens, b'[') {
                    let j = open.pop().unwrap();
                    tokens[i].partner = Some(j);
                    tokens[j].partner = Some(i);
                } else {
                    check.report(&t, Severity::Warning, "unmatched-bracket", "']' has no matching '['");
                }
            }
            (Kind::Op, b')') => {
                if top_is(&tokens, b'(') {
                    let j = open.pop().unwrap();
                    tokens[i].partner = Some(j);
                    tokens[j].partner = Some(i);
                    if !has_exit[j] {
                        check.report(&tokens[j].clone(), Severity::Warning, "endless-loop",
                            "this loop has no '^' to leave it, so it never ends");
                    }
                } else {
                    check.report(&t, Severity::Error, "unmatched-loop", "')' has no matching '('");
                }
            }
            (Kind::Op, b'^') => match open.iter().rev().copied().find(|&j| tokens[j].is_op(b'(')) {
                Some(j) => {
                    tokens[i].partner = Some(j);
                    has_exit[j] = true;
                }
                None => check.report(&t, Severity::Error, "exit-outside-loop", "'^' is not inside a loop"),
            },
            (Kind::Op, b',') => {
                if top_is_call {
                    let j = top.unwrap();
                    commas[j] += 1;
                    tokens[i].partner = Some(j);
                    tokens[i].arg = commas[j];
                } else {
                    check.report(&t, Severity::Error, "stray-comma",
                        "',' only separates the arguments of a macro call");
                }
            }
            (Kind::Op, b';') => {
                if top_is_call {
                    let j = open.pop().unwrap();
                    tokens[i].partner = Some(j);
                    tokens[j].partner = Some(i);
                } else {
                    check.report(&t, Severity::Error, "stray-semicolon", "';' has no macro call to end");
                }
            }
            (Kind::Op, b'@') => {
                if in_macro {
                    // Returning also abandons any loops still open.
                    for &j in &open {
                        has_exit[j] = true;
                    }
                } else {
                    check.report(&t, Severity::Error, "return-outside-macro", "'@' is not inside a macro");
                }
            }
            (Kind::Param, letter) => {
                if !in_macro {
                    check.report(&t, Severity::Error, "parameter-outside-macro",
                        format!("%{} is not inside a macro", letter as char));
                }
            }
            (Kind::BadCall, _) => check.report(&t, Severity::Error, "call-without-name",
                "'#' must be followed by the letter of a macro, A to Z"),
            (Kind::BadParam, _) => check.report(&t, Severity::Error, "parameter-without-name",
                "'%' must be followed by a letter: %A is the first argument, %B the second"),
            (Kind::Dollar, _) => check.report(&t, Severity::Warning, "lone-dollar",
                "a single '$' ends the program here: write '$$' to end it, or '$A' to define a macro"),
            (Kind::Str, _) if !t.terminated => check.report(&t, Severity::Error, "unclosed-string",
                "this string is never closed by a second '\"'"),
            (Kind::Unknown, _) => {
                let text = check.text(&t);
                let message = if text.bytes().all(|b| b.is_ascii_lowercase()) {
                    format!("'{text}' is ignored: variables and macros are the capital letters A to Z")
                } else if text.len() == 1 && b"<>\\{}&:~_|`".contains(&text.as_bytes()[0]) {
                    format!("'{text}' is ignored: it is not part of the 1979 language, which has no comparison operators")
                } else {
                    format!("'{text}' is not a Mouse instruction and is ignored")
                };
                check.report(&t, Severity::Warning, "ignored-text", message);
            }
            _ => {}
        }
    }

    if !ended {
        let upto = tokens.len();
        let mut region = regions.pop().unwrap();
        close_region(&mut check, &tokens, &mut open, &mut region, src.len(), upto);
        regions.push(region);
        if let Some(last) = tokens.iter().rev().find(|t| t.kind != Kind::Comment) {
            check.report(&last.clone(), Severity::Warning, "no-end", "the program has no closing '$$'");
        }
    }

    // Definitions: the interpreter keeps the last one for each letter.
    let mut definitions = [None; 26];
    for (i, t) in tokens.iter().enumerate() {
        if t.kind != Kind::Def {
            continue;
        }
        let slot = &mut definitions[(t.ch - b'A') as usize];
        if let Some(earlier) = *slot {
            let earlier: &Token = &tokens[earlier];
            check.diagnostics.push(Diagnostic {
                start: earlier.start,
                end: earlier.end,
                severity: Severity::Warning,
                code: "redefined",
                message: format!("macro ${} is defined again further down, and the later definition is the one used", t.ch as char),
                unnecessary: true,
            });
        }
        *slot = Some(i);
    }
    let mut called = [false; 26];
    for t in tokens.iter().filter(|t| t.kind == Kind::Call) {
        let letter = (t.ch - b'A') as usize;
        called[letter] = true;
        if definitions[letter].is_none() {
            check.report(t, Severity::Warning, "undefined-macro",
                format!("macro ${} is not defined, so this call does nothing", t.ch as char));
        }
    }
    for (letter, def) in definitions.iter().enumerate() {
        if let (Some(def), false) = (def, called[letter]) {
            let t = &tokens[*def];
            check.diagnostics.push(Diagnostic {
                start: t.start,
                end: t.end,
                severity: Severity::Hint,
                code: "unused-macro",
                message: format!("macro ${} is never called", t.ch as char),
                unnecessary: true,
            });
        }
    }

    let mut diagnostics = check.diagnostics;
    diagnostics.sort_by_key(|d| (d.start, d.end));
    Analysis { tokens, regions, diagnostics, definitions }
}

impl Analysis {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    /// The token under a cursor at `offset`. A cursor just after a token
    /// counts as on it when nothing else starts there.
    pub fn token_at(&self, offset: usize) -> Option<usize> {
        let i = self.tokens.partition_point(|t| t.end <= offset);
        match self.tokens.get(i) {
            Some(t) if t.start <= offset => Some(i),
            _ if i > 0 && self.tokens[i - 1].end == offset => Some(i - 1),
            _ => None,
        }
    }

    /// The definition the interpreter uses for a macro letter.
    pub fn definition(&self, letter: u8) -> Option<usize> {
        self.definitions[(letter - b'A') as usize]
    }

    /// The body region of a macro.
    pub fn body(&self, letter: u8) -> Option<&Region> {
        let def = self.definition(letter)?;
        self.regions.iter().find(|r| r.def == Some(def))
    }

    /// How many parameters a macro uses: 2 if its body mentions %B but
    /// nothing later.
    pub fn arity(&self, letter: u8) -> usize {
        self.body(letter).map_or(0, |region| {
            self.tokens[region.tokens.clone()]
                .iter()
                .filter(|t| t.kind == Kind::Param)
                .map(|t| (t.ch - b'A') as usize + 1)
                .max()
                .unwrap_or(0)
        })
    }

    /// Every definition of and call to a macro, as token indexes.
    pub fn macro_uses(&self, letter: u8) -> Vec<usize> {
        (0..self.tokens.len())
            .filter(|&i| matches!(self.tokens[i].kind, Kind::Def | Kind::Call) && self.tokens[i].ch == letter)
            .collect()
    }

    /// Every mention of a variable in one region. Each macro call has its own
    /// variables, so the same letter elsewhere is a different variable.
    pub fn variable_uses(&self, letter: u8, region: usize) -> Vec<usize> {
        (0..self.tokens.len())
            .filter(|&i| {
                let t = &self.tokens[i];
                t.kind == Kind::Variable && t.ch == letter && t.region == region
            })
            .collect()
    }

    /// The comment that describes a macro: the one after it on the line of
    /// its `$A`, or else the comment lines directly above.
    pub fn macro_doc(&self, source: &str, letter: u8) -> Option<String> {
        let def = self.definition(letter)?;
        let src = source.as_bytes();
        let line_of = |offset: usize| src[..offset].iter().filter(|&&b| b == b'\n').count();
        let strip = |t: &Token| source[t.start + 1..t.end].trim().to_string();
        let line = line_of(self.tokens[def].start);

        let same_line = self.tokens[def + 1..]
            .iter()
            .take_while(|t| line_of(t.start) == line)
            .find(|t| t.kind == Kind::Comment);
        if let Some(comment) = same_line {
            return Some(strip(comment)).filter(|s| !s.is_empty());
        }

        let mut lines = Vec::new();
        let mut expect = line;
        for t in self.tokens[..def].iter().rev() {
            if t.kind != Kind::Comment || expect == 0 || line_of(t.start) != expect - 1 {
                break;
            }
            // A comment after code on its line describes that code, not us.
            let line_start = src[..t.start].iter().rposition(|&b| b == b'\n').map_or(0, |p| p + 1);
            if !src[line_start..t.start].iter().all(|&b| is_space(b)) {
                break;
            }
            lines.push(strip(t));
            expect -= 1;
        }
        lines.reverse();
        Some(lines.join("\n")).filter(|s| !s.trim().is_empty())
    }
}

/// Converts a byte offset to a 1-based line and column, counting characters.
pub fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset.min(source.len())];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let start = before.rfind('\n').map_or(0, |p| p + 1);
    (line, before[start..].chars().count() + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(src: &str) -> Vec<&'static str> {
        analyse(src).diagnostics.iter().map(|d| d.code).collect()
    }

    #[test]
    fn a_correct_program_has_no_diagnostics() {
        assert!(codes("N 5 =\n( N. ^ N. ! N N. 1 - = )\n\"done!\"\n$$").is_empty());
        assert!(codes("#A,#B;;\n\n$A Q 1 = %A Q. ! @\n$B Q 33 = @\n$$").is_empty());
    }

    #[test]
    fn tokens_cover_calls_definitions_and_parameters() {
        let a = analyse("#M,7,3; !\n$M %A %B - @ ' subtract\n$$\nignored input");
        let kinds: Vec<Kind> = a.tokens.iter().map(|t| t.kind).collect();
        assert_eq!(kinds, [
            Kind::Call, Kind::Op, Kind::Number, Kind::Op, Kind::Number, Kind::Op, Kind::Op,
            Kind::Def, Kind::Param, Kind::Param, Kind::Op, Kind::Op, Kind::Comment,
            Kind::End, Kind::Trailer,
        ]);
        assert_eq!(a.arity(b'M'), 2);
        assert_eq!(a.macro_uses(b'M').len(), 2);
        assert_eq!(a.tokens[2].arg, 1);
        assert_eq!(a.tokens[4].arg, 2);
        assert_eq!(a.tokens[0].partner, Some(5));
    }

    #[test]
    fn brackets_inside_strings_and_comments_are_not_counted() {
        assert!(codes("1 [ \"a ] in a string\" ] ' and a ( in a comment\n$$").is_empty());
    }

    #[test]
    fn unbalanced_brackets_loops_and_calls() {
        assert_eq!(codes("1 [ 2 !\n$$"), ["unclosed-bracket"]);
        assert_eq!(codes("1 ] 2 !\n$$"), ["unmatched-bracket"]);
        assert_eq!(codes("( 1 ^ 2 !\n$$"), ["unclosed-loop"]);
        assert_eq!(codes("1 ) 2 !\n$$"), ["unmatched-loop"]);
        assert_eq!(codes("#A,1 !\n$A %A @\n$$"), ["unclosed-call"]);
        assert_eq!(codes("1 , 2 ;\n$$"), ["stray-comma", "stray-semicolon"]);
        assert_eq!(codes("\"never closed\n$$"), ["unclosed-string", "no-end"]);
    }

    #[test]
    fn things_that_only_make_sense_in_a_macro_or_a_loop() {
        assert_eq!(codes("%A ! @ 1 ^\n$$"), ["parameter-outside-macro", "return-outside-macro", "exit-outside-loop"]);
        assert!(codes("#A,1;\n$A ( %A ^ 1 @ ) @\n$$").is_empty());
    }

    #[test]
    fn macros_undefined_unused_redefined_and_unreturned() {
        assert_eq!(codes("#A;\n$$"), ["undefined-macro"]);
        assert_eq!(codes("1 !\n$A 2 ! @\n$$"), ["unused-macro"]);
        assert_eq!(codes("#A;\n$A 1 ! @\n$A 2 ! @\n$$"), ["redefined"]);
        assert_eq!(codes("#A;\n$A 1 !\n$$"), ["no-return"]);
    }

    #[test]
    fn loops_with_no_exit_and_programs_with_no_end() {
        assert_eq!(codes("( 1 ! )\n$$"), ["endless-loop"]);
        assert_eq!(codes("1 !"), ["no-end"]);
        assert!(codes("").is_empty());
    }

    #[test]
    fn ignored_text_is_reported_once_per_word() {
        let a = analyse("x 5 =\nN 1 < !\n$$");
        let messages: Vec<&str> = a.diagnostics.iter().map(|d| d.message.as_str()).collect();
        assert_eq!(messages.len(), 2);
        assert!(messages[0].contains("capital letters"));
        assert!(messages[1].contains("comparison"));
        assert!(!a.has_errors());
        assert_eq!(codes("hello world\n$$"), ["ignored-text", "ignored-text"]);
    }

    #[test]
    fn a_macro_is_described_by_its_comment() {
        let src = "#M,7,3; #Z,0;\n' Z(a) is 1 when a is 0,\n' and 0 otherwise.\n$Z %A [ 0 @ ] 1 @\n$M %A %B - @   ' M(a, b) = a - b\n$$";
        let a = analyse(src);
        assert_eq!(a.macro_doc(src, b'M').as_deref(), Some("M(a, b) = a - b"));
        assert_eq!(a.macro_doc(src, b'Z').as_deref(), Some("Z(a) is 1 when a is 0,\nand 0 otherwise."));
    }

    #[test]
    fn variables_are_separate_in_each_region() {
        let a = analyse("Q 1 = #A; Q. !\n$A Q 2 = @\n$$");
        let main = a.variable_uses(b'Q', 0);
        assert_eq!(main.len(), 2);
        assert_eq!(a.variable_uses(b'Q', a.tokens[a.definition(b'A').unwrap()].region).len(), 1);
    }

    #[test]
    fn token_at_finds_the_token_under_or_just_before_the_cursor() {
        let a = analyse("#M,7; N.\n$M %A @\n$$");
        assert_eq!(a.tokens[a.token_at(0).unwrap()].kind, Kind::Call);
        assert_eq!(a.tokens[a.token_at(2).unwrap()].kind, Kind::Op);
        assert_eq!(a.tokens[a.token_at(7).unwrap()].kind, Kind::Op);
        assert_eq!(a.token_at(5), Some(3));
        assert_eq!(line_column("ab\ncd", 4), (2, 2));
    }

    #[test]
    fn positions_survive_text_outside_ascii() {
        let src = "\"café — ok\" x\n$$";
        let a = analyse(src);
        assert_eq!(a.diagnostics.len(), 1);
        assert_eq!(&src[a.diagnostics[0].start..a.diagnostics[0].end], "x");
        assert_eq!(line_column(src, a.diagnostics[0].start), (1, 13));
    }
}
