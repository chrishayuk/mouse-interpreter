//! MOUSE - A Language for Microcomputers (interpreter)
//!
//! This is the language Peter Grogono described in "Mouse: A Language for
//! Microcomputers", BYTE, July 1979, pp. 198-220, and follows the Pascal
//! interpreter printed there as listing 6. It includes the four fixes from
//! Tom Lane's "Comment and Correction for Mouse", BYTE, June 1980, all of
//! which Grogono accepted in his reply:
//!
//! 1. A PARAM frame *increments* PARBAL when searching for the macro call a
//!    formal parameter belongs to (line 176 was misprinted with a minus).
//! 2. SKIP steps over quoted strings, so `[ "a ] in a string" ]` works.
//! 3. The scan for an actual parameter steps over quoted strings and over
//!    adjacent macro calls inside earlier parameters.
//! 4. Local variables are allocated from LASTUSED, not OFFSET, so a macro
//!    called from inside an actual parameter gets storage of its own.
//!
//! See README.md for the places where this interpreter goes beyond the
//! article (comments, error reporting, no fixed array sizes).

mod analysis;

use std::collections::VecDeque;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::ExitCode;

/// Each macro call gets its own block of 26 variables.
const VARS_PER_FRAME: i32 = 26;

/// Depth of nested loops, macro calls and parameters before giving up. The
/// article's interpreter allowed 20; this is only a guard against runaway
/// recursion.
const MAX_STACK_DEPTH: usize = 100_000;

#[derive(Debug)]
enum MouseError {
    Io(io::Error),
    Runtime { msg: String, pos: usize },
}

impl fmt::Display for MouseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MouseError::Io(e) => write!(f, "{e}"),
            MouseError::Runtime { msg, pos } => write!(f, "{msg} (program offset {pos})"),
        }
    }
}

impl From<io::Error> for MouseError {
    fn from(e: io::Error) -> Self {
        MouseError::Io(e)
    }
}

type Result<T> = std::result::Result<T, MouseError>;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tag {
    Macro,
    Param,
    Loop,
}

#[derive(Clone, Copy, Debug)]
struct Frame {
    tag: Tag,
    pos: usize,
    off: i32,
}

/// A loaded program: the source with comments and insignificant whitespace
/// removed, plus the start of each macro body.
struct Program {
    code: Vec<u8>,
    definitions: [Option<usize>; 26],
}

impl Program {
    /// Reads a program up to and including the terminating `$$`. Nothing past
    /// the `$$` is consumed, so a program piped on stdin can be followed by
    /// its own input.
    fn load(input: impl Read) -> io::Result<Self> {
        let mut code = Vec::new();
        let mut definitions = [None; 26];
        let mut in_string = false;
        let mut in_comment = false;
        let mut last = b' ';

        for byte in input.bytes() {
            let this = byte?;

            if in_comment {
                if this == b'\n' {
                    in_comment = false;
                    last = this;
                }
                continue;
            }
            if this == b'\'' && !in_string {
                // A comment still separates two numbers.
                if last.is_ascii_digit() {
                    code.push(b' ');
                }
                in_comment = true;
                continue;
            }

            if !in_string && this.is_ascii_uppercase() && last == b'$' {
                definitions[(this - b'A') as usize] = Some(code.len() + 1);
            }
            if this == b'"' {
                in_string = !in_string;
            }
            // Whitespace is kept inside strings, and after a digit so that
            // "12 34" stays two numbers.
            let is_space = matches!(this, b' ' | b'\t' | b'\n' | b'\r');
            if in_string || !is_space || last.is_ascii_digit() {
                code.push(this);
            }
            if !in_string && this == b'$' && last == b'$' {
                return Ok(Program { code, definitions });
            }
            last = this;
        }

        // End of input without `$$`: terminate the program anyway.
        code.push(b'$');
        Ok(Program { code, definitions })
    }
}

struct Mouse {
    prog: Program,
    calstack: Vec<i32>,
    data: Vec<i32>,
    stack: Vec<Frame>,
    chpos: usize,
    /// Start of the variables that the letters A-Z currently refer to.
    offset: i32,
    /// Highest variable address handed out so far (Lane's LASTUSED).
    last_used: i32,
    ch: u8,
    pending_input: VecDeque<String>,
}

impl Mouse {
    fn new(prog: Program) -> Self {
        Mouse {
            prog,
            calstack: Vec::with_capacity(256),
            data: vec![0; 256],
            stack: Vec::with_capacity(256),
            chpos: 0,
            offset: 0,
            last_used: VARS_PER_FRAME,
            ch: 0,
            pending_input: VecDeque::new(),
        }
    }

    fn error<T>(&self, msg: impl Into<String>) -> Result<T> {
        Err(MouseError::Runtime {
            msg: msg.into(),
            pos: self.chpos.saturating_sub(1),
        })
    }

    fn nextchar(&mut self) -> Result<()> {
        match self.prog.code.get(self.chpos) {
            Some(&c) => {
                self.ch = c;
                self.chpos += 1;
                Ok(())
            }
            None => self.error("ran off the end of the program (unbalanced brackets or quotes?)"),
        }
    }

    /// Reads the letter following `#` or `%` and returns its index.
    fn next_letter(&mut self, after: char) -> Result<usize> {
        self.nextchar()?;
        if self.ch.is_ascii_uppercase() {
            Ok((self.ch - b'A') as usize)
        } else {
            self.error(format!(
                "expected a letter A-Z after '{after}', found '{}'",
                self.ch as char
            ))
        }
    }

    fn pushcal(&mut self, datum: i32) {
        self.calstack.push(datum);
    }

    fn popcal(&mut self) -> Result<i32> {
        match self.calstack.pop() {
            Some(v) => Ok(v),
            None => self.error(format!("stack underflow at '{}'", self.ch as char)),
        }
    }

    fn push(&mut self, tag: Tag) -> Result<()> {
        if self.stack.len() >= MAX_STACK_DEPTH {
            return self.error("control stack overflow (runaway recursion?)");
        }
        self.stack.push(Frame {
            tag,
            pos: self.chpos,
            off: self.offset,
        });
        Ok(())
    }

    fn pop(&mut self) -> Result<()> {
        match self.stack.pop() {
            Some(frame) => {
                self.chpos = frame.pos;
                self.offset = frame.off;
                Ok(())
            }
            None => self.error(format!(
                "'{}' with no matching loop, macro call or parameter",
                self.ch as char
            )),
        }
    }

    /// Steps over the rest of a quoted string whose opening quote has just
    /// been read.
    fn skip_string(&mut self) -> Result<()> {
        loop {
            self.nextchar()?;
            if self.ch == b'"' {
                return Ok(());
            }
        }
    }

    /// Skips forward to the `rch` that balances an already-consumed `lch`,
    /// ignoring anything inside quoted strings.
    fn skip(&mut self, lch: u8, rch: u8) -> Result<()> {
        let mut cnt = 1;
        while cnt != 0 {
            self.nextchar()?;
            if self.ch == b'"' {
                self.skip_string()?;
            } else if self.ch == lch {
                cnt += 1;
            } else if self.ch == rch {
                cnt -= 1;
            }
        }
        Ok(())
    }

    /// Variable addresses start at 1: in the main program A is 1 and Z is 26.
    fn address(&self, addr: i32) -> Result<usize> {
        match usize::try_from(addr) {
            Ok(a) if a >= 1 => Ok(a),
            _ => self.error(format!("invalid variable address {addr}")),
        }
    }

    /// Reads the next whitespace-separated integer from `input`.
    fn read_number(&mut self, input: &mut dyn BufRead) -> Result<i32> {
        loop {
            if let Some(token) = self.pending_input.pop_front() {
                return match token.parse() {
                    Ok(n) => Ok(n),
                    Err(_) => self.error(format!("'?' expected a number, got '{token}'")),
                };
            }
            let mut line = String::new();
            if input.read_line(&mut line)? == 0 {
                return self.error("'?' reached the end of input");
            }
            self.pending_input
                .extend(line.split_whitespace().map(str::to_owned));
        }
    }

    fn run(&mut self, input: &mut dyn BufRead, out: &mut dyn Write) -> Result<()> {
        loop {
            self.nextchar()?;
            match self.ch {
                b'0'..=b'9' => {
                    let mut n: i32 = 0;
                    while self.ch.is_ascii_digit() {
                        n = n.wrapping_mul(10).wrapping_add((self.ch - b'0') as i32);
                        self.nextchar()?;
                    }
                    self.pushcal(n);
                    self.chpos -= 1;
                }
                b'A'..=b'Z' => {
                    self.pushcal((self.ch - b'A') as i32 + 1 + self.offset);
                }
                b'?' => {
                    out.flush()?;
                    let n = self.read_number(input)?;
                    self.pushcal(n);
                }
                b'!' => {
                    let v = self.popcal()?;
                    write!(out, "{v}")?;
                }
                b'+' => {
                    let b = self.popcal()?;
                    let a = self.popcal()?;
                    self.pushcal(a.wrapping_add(b));
                }
                b'-' => {
                    let b = self.popcal()?;
                    let a = self.popcal()?;
                    self.pushcal(a.wrapping_sub(b));
                }
                b'*' => {
                    let b = self.popcal()?;
                    let a = self.popcal()?;
                    self.pushcal(a.wrapping_mul(b));
                }
                b'/' => {
                    let b = self.popcal()?;
                    let a = self.popcal()?;
                    if b == 0 {
                        return self.error("division by zero");
                    }
                    self.pushcal(a.wrapping_div(b));
                }
                b'.' => {
                    let addr = self.popcal()?;
                    let index = self.address(addr)?;
                    self.pushcal(self.data.get(index).copied().unwrap_or(0));
                }
                b'=' => {
                    let value = self.popcal()?;
                    let addr = self.popcal()?;
                    let index = self.address(addr)?;
                    if index >= self.data.len() {
                        self.data.resize(index + 1, 0);
                    }
                    self.data[index] = value;
                }
                b'"' => loop {
                    self.nextchar()?;
                    match self.ch {
                        b'!' => out.write_all(b"\n")?,
                        b'"' => break,
                        c => out.write_all(&[c])?,
                    }
                },
                b'[' => {
                    if self.popcal()? <= 0 {
                        self.skip(b'[', b']')?;
                    }
                }
                b'(' => self.push(Tag::Loop)?,
                b'^' => {
                    if self.popcal()? <= 0 {
                        self.pop()?;
                        self.skip(b'(', b')')?;
                    }
                }
                b')' => match self.stack.last() {
                    Some(frame) => self.chpos = frame.pos,
                    None => return self.error("')' with no matching '('"),
                },
                b'#' => {
                    let name = self.next_letter('#')?;
                    match self.prog.definitions[name] {
                        Some(def) => {
                            self.push(Tag::Macro)?;
                            self.chpos = def;
                            self.offset = self.last_used;
                            self.last_used += VARS_PER_FRAME;
                        }
                        None => self.skip(b'#', b';')?,
                    }
                }
                b'@' => {
                    // Return to the call, abandoning any loops still open
                    // inside the macro, then step over its actual parameters.
                    loop {
                        match self.stack.last().map(|frame| frame.tag) {
                            Some(Tag::Macro) => break,
                            Some(_) => {
                                self.stack.pop();
                            }
                            None => return self.error("'@' outside a macro"),
                        }
                    }
                    self.pop()?;
                    self.last_used -= VARS_PER_FRAME;
                    self.skip(b'#', b';')?;
                }
                b'%' => self.parameter()?,
                b',' | b';' => self.pop()?,
                // ' ', ']', '$' and anything unrecognised do nothing.
                _ => {}
            }
            if self.ch == b'$' {
                break;
            }
        }
        out.flush()?;
        Ok(())
    }

    /// `%A`: evaluate the first actual parameter of the current macro call,
    /// `%B` the second, and so on. Execution jumps back into the caller's
    /// argument list, in the caller's environment, until the `,` or `;` that
    /// ends the argument pops us back here.
    fn parameter(&mut self) -> Result<()> {
        let mut parnum = self.next_letter('%')? as i32 + 1;
        self.push(Tag::Param)?;

        // Find the macro call this parameter belongs to. A PARAM frame on the
        // way down means we are already evaluating an argument on behalf of
        // an inner call, so that frame and its macro frame are skipped.
        let mut parbal = 1;
        let mut temp = self.stack.len() - 1;
        while parbal != 0 {
            if temp == 0 {
                self.stack.pop();
                return self.error("'%' used outside a macro");
            }
            temp -= 1;
            match self.stack[temp].tag {
                Tag::Macro => parbal -= 1,
                Tag::Param => parbal += 1,
                Tag::Loop => {}
            }
        }
        self.chpos = self.stack[temp].pos;
        self.offset = self.stack[temp].off;

        // Step over the arguments before the one we want: each is introduced
        // by a comma. Strings and nested calls may contain commas of their own.
        while parnum != 0 {
            self.nextchar()?;
            match self.ch {
                b'"' => self.skip_string()?,
                b'#' => {
                    self.skip(b'#', b';')?;
                    // The ';' that ended the nested call is not ours.
                    self.ch = b' ';
                }
                b',' => parnum -= 1,
                b';' => parnum = 0,
                _ => {}
            }
        }
        // The call has fewer arguments than this: the parameter is empty.
        if self.ch == b';' {
            self.pop()?;
        }
        Ok(())
    }
}

/// `mouse --check FILE...`: report mistakes without running anything.
/// Returns false if any file has an error.
fn check_files(paths: &[String]) -> io::Result<bool> {
    let mut ok = true;
    for path in paths {
        // Read through `take` so that the WASI build needs no file metadata
        // call, which the browser's small WASI host does not provide.
        let mut source = String::new();
        File::open(path)
            .and_then(|file| file.take(u64::MAX).read_to_string(&mut source))
            .map_err(|e| io::Error::new(e.kind(), format!("cannot read {path}: {e}")))?;
        let result = analysis::analyse(&source);
        for d in &result.diagnostics {
            let (line, column) = analysis::line_column(&source, d.start);
            println!("{path}:{line}:{column}: {}: {}", d.severity.name(), d.message);
        }
        ok &= !result.has_errors();
    }
    Ok(ok)
}

fn run_cli() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.get(1).map(String::as_str) == Some("--check") {
        if args.len() < 3 {
            return Err(MouseError::Io(io::Error::other("usage: mouse --check FILE...")));
        }
        if !check_files(&args[2..])? {
            std::process::exit(1);
        }
        return Ok(());
    }
    let stdin = io::stdin();
    let mut input = stdin.lock();

    let prog = match args.get(1) {
        Some(path) => {
            let file = File::open(path).map_err(|e| {
                MouseError::Io(io::Error::new(e.kind(), format!("cannot load {path}: {e}")))
            })?;
            Program::load(BufReader::new(file))?
        }
        None => Program::load(&mut input)?,
    };

    let mut out = io::stdout().lock();
    let result = Mouse::new(prog).run(&mut input, &mut out);
    // Show whatever the program printed before any error message.
    out.flush()?;
    result
}

fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        // Output piped into something that stopped reading, e.g. `| head`.
        Err(MouseError::Io(e)) if e.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mouse: error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_with(src: &str, input: &str) -> Result<String> {
        let prog = Program::load(src.as_bytes())?;
        let mut out = Vec::new();
        Mouse::new(prog).run(&mut input.as_bytes(), &mut out)?;
        Ok(String::from_utf8(out).unwrap())
    }

    fn run(src: &str) -> String {
        run_with(src, "").unwrap()
    }

    #[test]
    fn prints_strings_with_bang_newlines() {
        assert_eq!(run("\"hello world!\"$$"), "hello world\n");
        assert_eq!(run("\"Hello!\"\n\"World!\"$$"), "Hello\nWorld\n");
    }

    #[test]
    fn keeps_literal_newlines_inside_strings() {
        assert_eq!(run("\"a\nb!\"$$"), "a\nb\n");
    }

    #[test]
    fn apostrophe_is_a_comment_outside_strings_only() {
        assert_eq!(run("1 ' ignored \"x\" $$\n 2 + ! $$"), "3");
        assert_eq!(run("\"don't!\"$$"), "don't\n");
        assert_eq!(run("12'c\n34 + ! $$"), "46");
    }

    #[test]
    fn arithmetic_uses_left_to_right_operands() {
        assert_eq!(run("7 2 - ! $$"), "5");
        assert_eq!(run("7 2 / ! $$"), "3");
        assert_eq!(run("3 4 + 5 * ! $$"), "35");
    }

    #[test]
    fn numbers_on_separate_lines_stay_separate() {
        assert_eq!(run("12\n34 + ! $$"), "46");
    }

    #[test]
    fn variables() {
        assert_eq!(run("X 5 = Y X. 2 * = Y. ! $$"), "10");
        assert_eq!(run("K 9 = K. ! $$"), "9");
    }

    #[test]
    fn conditionals() {
        assert_eq!(run("1 [ \"yes\" ] 0 [ \"no\" ] $$"), "yes");
        assert_eq!(run("0 [ 1 [ \"a\" ] \"b\" ] \"c\" $$"), "c");
    }

    #[test]
    fn loops() {
        assert_eq!(run("N 3 = ( N. ^ N. ! \" \" N N. 1 - = ) $$"), "3 2 1 ");
    }

    #[test]
    fn nested_loops() {
        let src = "I 2 = ( I. ^ J 2 = ( J. ^ I. ! J. ! \" \" J J. 1 - = ) I I. 1 - = ) $$";
        assert_eq!(run(src), "22 21 12 11 ");
    }

    #[test]
    fn reads_input() {
        assert_eq!(run_with("? ? + ! $$", "3 4\n").unwrap(), "7");
        assert_eq!(run_with("? ? + ! $$", "3\n\n4\n").unwrap(), "7");
        assert_eq!(run_with("? ! $$", "-12\n").unwrap(), "-12");
    }

    #[test]
    fn fibonacci() {
        let src = "N ? = A 0 = B 1 =
                   ( N. ^ A. ! \" \" T A. B. + = A B. = B T. = N N. 1 - = ) $$";
        assert_eq!(run_with(src, "8\n").unwrap(), "0 1 1 2 3 5 8 13 ");
    }

    #[test]
    fn macro_without_parameters() {
        assert_eq!(run("#H; #H; $\n$H \"hi!\" @\n$$"), "hi\nhi\n");
    }

    #[test]
    fn macro_parameters() {
        assert_eq!(run("#D,5; ! $\n$D %A 2 * @\n$$"), "10");
        assert_eq!(run("#S,10,3; ! $\n$S %A %B - @\n$$"), "7");
    }

    #[test]
    fn macro_locals_do_not_clobber_the_caller() {
        let src = "X 1 = #M; X. ! $\n$M X 99 = @\n$$";
        assert_eq!(run(src), "1");
    }

    #[test]
    fn arguments_are_evaluated_in_the_callers_environment() {
        let src = "X 4 = #D,X.; ! $\n$D X 100 = %A 2 * @\n$$";
        assert_eq!(run(src), "8");
    }

    #[test]
    fn parameter_passed_through_a_nested_call() {
        let src = "#X,7; ! $\n$X #Y,%A; @\n$Y %A 1 + @\n$$";
        assert_eq!(run(src), "8");
    }

    #[test]
    fn macro_call_as_an_argument() {
        let src = "#D,#D,3;; ! $\n$D %A 2 * @\n$$";
        assert_eq!(run(src), "12");
    }

    #[test]
    fn recursive_macro() {
        // Factorial: F(n) = n <= 1 ? 1 : n * F(n - 1)
        let src = "#F,5; ! $
                   $F %A 1 - [ %A #F,%A 1 -; * @ ] 1 @
                   $$";
        assert_eq!(run(src), "120");
    }

    #[test]
    fn deep_recursion_grows_variable_storage() {
        // Each level stores into its own local; 50 levels needs 1300+ cells.
        let src = "#C,50; ! $
                   $C N %A = N. 1 - [ #C,N. 1 -; 1 + @ ] 1 @
                   $$";
        assert_eq!(run(src), "50");
    }

    // ---- Behaviour taken from the July 1979 article ----

    #[test]
    fn addresses_start_at_one() {
        assert_eq!(run("A ! \" \" T ! $$"), "1 20");
    }

    #[test]
    fn article_stack_as_anonymous_variable() {
        assert_eq!(run("A A 5 = . A 6 = ! $$"), "5");
    }

    #[test]
    fn article_swap_without_a_temporary() {
        assert_eq!(run("X 1 = Y 2 = X Y. Y X. = = X. ! Y. ! $$"), "21");
    }

    #[test]
    fn article_letters_as_arrays() {
        assert_eq!(run("A 7 = B A. = A. ! B. ! A 1 + . ! $$"), "777");
    }

    #[test]
    fn article_factorial_programs() {
        let src = "N ? = F 1 = ( N. ^ F F. N. * = N N. 1 - = ) F. ! $$";
        assert_eq!(run_with(src, "5\n").unwrap(), "120");
        let src = "N ? = 1 ( N. ^ N. * N N. 1 - = ) ! $$";
        assert_eq!(run_with(src, "6\n").unwrap(), "720");
    }

    #[test]
    fn article_call_by_value_and_reference() {
        assert_eq!(run("#P,3; A 5 = #P,A.; #P,A; $P %A ! @ $$"), "351");
        let src = "#A,2,3,X; #A,3,4,Y; #A,X.,Y.,Z; Z. ! $A %C %A %B + = @ $$";
        assert_eq!(run(src), "12");
    }

    #[test]
    fn article_function_macros() {
        // As corrected by Grogono in June 1980.
        let src = "#F,5; #F,6; + ! $F N %A = 1 ( N. ^ N. * N N. 1 - = ) @ $$";
        assert_eq!(run(src), "840");
        let src = "#F,5; ! $F %A [ %A #F,%A 1 -; * @ ] 1 @ $$";
        assert_eq!(run(src), "120");
    }

    #[test]
    fn article_missing_and_extra_parameters() {
        let src = "#T; #T,\"A\"; #T,\"A\",\"B\"; #T,\"A\",\"B\",\"C\"; #U;
                   $T \"<\" %A %B \">!\" @ $$";
        assert_eq!(run(src), "<>\n<A>\n<AB>\n<AB>\n");
    }

    #[test]
    fn article_figure_one() {
        assert_eq!(run("#A,3; $A #B,%A; @ $B %A ! @ $$"), "3");
    }

    // ---- Tom Lane's corrections, BYTE June 1980 ----

    #[test]
    fn lane_skip_ignores_brackets_in_strings() {
        assert_eq!(run("0 [ \"PRINT A BRACKET ] HERE\" ] \"ok\" $$"), "ok");
        assert_eq!(run("#U,\"a;b\"; \"ok\" $$"), "ok");
    }

    #[test]
    fn lane_adjacent_calls_inside_a_parameter() {
        let src = "#A,#B,1;#C,2;,34; $A %B ! @ $B @ $C @ $$";
        assert_eq!(run(src), "34");
    }

    #[test]
    fn lane_commas_in_string_parameters() {
        assert_eq!(run("#A,\"x,y;\",7; $A %B ! %A @ $$"), "7x,y;");
    }

    #[test]
    fn lane_macro_called_from_a_parameter_gets_its_own_locals() {
        let src = "#A,#B;; $A Q 1 = %A Q. ! @ $B Q 33 = @ $$";
        assert_eq!(run(src), "1");
    }

    // ---- Beyond the article ----

    #[test]
    fn return_from_inside_a_loop() {
        let src = "#F; \"done\" $F N 5 = ( N. ! N N. 1 - = 3 N. - [ @ ] ) @ $$";
        assert_eq!(run(src), "543done");
    }

    #[test]
    fn missing_argument_is_empty() {
        assert_eq!(run("3 #P; ! $\n$P %A @\n$$"), "3");
    }

    #[test]
    fn undefined_macro_call_is_skipped() {
        assert_eq!(run("#Q,1,2; \"ok\" $$"), "ok");
    }

    #[test]
    fn program_without_terminator_still_runs() {
        assert_eq!(run("\"hi\""), "hi");
    }

    #[test]
    fn loading_stops_at_the_terminator() {
        let mut src = "? ! $$ 42\n".as_bytes();
        let prog = Program::load(&mut src).unwrap();
        let mut out = Vec::new();
        Mouse::new(prog).run(&mut src, &mut out).unwrap();
        assert_eq!(out, b"42");
    }

    #[test]
    fn errors_instead_of_panicking() {
        for (src, expect) in [
            ("+ $$", "stack underflow"),
            ("1 0 / $$", "division by zero"),
            ("? $$", "end of input"),
            ("0 [ $$", "ran off the end"),
            (") $$", "no matching '('"),
            ("%A $$", "outside a macro"),
            ("@ $$", "outside a macro"),
            ("0 . $$", "invalid variable address"),
            ("#R; $R #R; @ $$", "control stack overflow"),
            ("#1; $$", "expected a letter"),
            ("0 1 - . $$", "invalid variable address"),
        ] {
            let err = run_with(src, "").unwrap_err().to_string();
            assert!(err.contains(expect), "{src:?} gave {err:?}");
        }
        let err = run_with("? $$", "abc\n").unwrap_err().to_string();
        assert!(err.contains("expected a number"), "{err}");
    }
}
