use std::fs::File;
use std::io::{self, BufRead, Read};
use std::env;

#[derive(Clone, Copy)]
enum TagType {
    Macro,
    Param,
    Loop,
}

struct Frame {
    tag: TagType,
    pos: usize,
    off: i32,
}

struct Mouse {
    prog: Vec<u8>,
    definitions: [usize; 26],
    calstack: Vec<i32>,
    data: [i32; 256],
    stack: Vec<Frame>,
    cal: usize,
    chpos: usize,
    level: usize,
    offset: i32,
    parnum: i32,
    parbal: i32,
    temp: i32,
    ch: u8,
}

impl Mouse {
    fn new() -> Self {
        Mouse {
            prog: Vec::with_capacity(5000),
            definitions: [0; 26],
            calstack: Vec::with_capacity(256),
            data: [0; 256],
            stack: Vec::with_capacity(256),
            cal: 0,
            chpos: 0,
            level: 0,
            offset: 0,
            parnum: 0,
            parbal: 0,
            temp: 0,
            ch: 0,
        }
    }

    fn num(&self, ch: u8) -> usize {
        (ch - b'A') as usize
    }

    fn val(&self, ch: u8) -> i32 {
        (ch - b'0') as i32
    }

    fn nextchar(&mut self) {
        self.ch = self.prog[self.chpos];
        self.chpos += 1;
    }

    fn pushcal(&mut self, datum: i32) {
        self.calstack.push(datum);
    }

    fn popcal(&mut self) -> i32 {
        self.calstack.pop().unwrap()
    }

    fn push(&mut self, tagval: TagType) {
        self.stack.push(Frame {
            tag: tagval,
            pos: self.chpos,
            off: self.offset,
        });
    }

    fn pop(&mut self) {
        if let Some(frame) = self.stack.pop() {
            self.chpos = frame.pos;
            self.offset = frame.off;
        }
    }

    fn skip(&mut self, lch: u8, rch: u8) {
        let mut cnt = 1;
        while cnt != 0 {
            self.nextchar();
            if self.ch == lch {
                cnt += 1;
            } else if self.ch == rch {
                cnt -= 1;
            }
        }
    }

    fn load(&mut self, input: impl Read) -> io::Result<()> {
        let mut reader = io::BufReader::new(input);
        let mut in_string = false;
        let mut last = b' ';

        for line in reader.lines() {
            let line = line?;
            for this in line.bytes() {
                if this == b'\'' {
                    break;  // Comment, skip rest of line
                } else {
                    if this >= b'A' && this <= b'Z' && last == b'$' {
                        self.definitions[self.num(this)] = self.prog.len() + 1;
                    }
                    if this == b'"' {
                        in_string = !in_string;
                    }
                    if !this.is_ascii_whitespace() || in_string || (last.is_ascii_digit()) {
                        self.prog.push(this);
                    }
                }
                last = this;
                if this == b'$' && last == b'$' {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    fn interpret(&mut self) {
        loop {
            self.nextchar();
            match self.ch {
                b' ' | b']' | b'$' => {}
                b'0'..=b'9' => {
                    self.temp = 0;
                    while self.ch.is_ascii_digit() {
                        self.temp = 10 * self.temp + self.val(self.ch);
                        self.nextchar();
                    }
                    self.pushcal(self.temp);
                    self.chpos -= 1;
                }
                b'A'..=b'Z' => {
                    self.pushcal(self.num(self.ch) as i32 + self.offset);
                }
                b'?' => {
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    self.temp = input.trim().parse().unwrap();
                    self.pushcal(self.temp);
                }
                b'!' => print!("{}", self.popcal()),
                b'+' => {
                    let a = self.popcal();
                    let b = self.popcal();
                    self.pushcal(a + b);
                }
                b'-' => {
                    let a = self.popcal();
                    let b = self.popcal();
                    self.pushcal(b - a);
                }
                b'*' => {
                    let a = self.popcal();
                    let b = self.popcal();
                    self.pushcal(a * b);
                }
                b'/' => {
                    let a = self.popcal();
                    let b = self.popcal();
                    self.pushcal(b / a);
                }
                b'.' => {
                    let index = self.popcal() as usize;
                    self.pushcal(self.data[index]);
                }
                b'=' => {
                    let value = self.popcal();
                    let index = self.popcal() as usize;
                    self.data[index] = value;
                }
                b'"' => {
                    loop {
                        self.nextchar();
                        match self.ch {
                            b'!' => println!(),
                            b'"' => break,
                            _ => print!("{}", self.ch as char),
                        }
                    }
                }
                b'[' => {
                    if self.popcal() <= 0 {
                        self.skip(b'[', b']');
                    }
                }
                b'(' => self.push(TagType::Loop),
                b'^' => {
                    if self.popcal() <= 0 {
                        self.pop();
                        self.skip(b'(', b')');
                    }
                }
                b')' => {
                    if let Some(frame) = self.stack.last() {
                        self.chpos = frame.pos;
                    }
                }
                b'#' => {
                    self.nextchar();
                    let def = self.definitions[self.num(self.ch)];
                    if def > 0 {
                        self.push(TagType::Macro);
                        self.chpos = def;
                        self.offset += 26;
                    } else {
                        self.skip(b'#', b';');
                    }
                }
                b'@' | b'}' => {
                    self.pop();
                    self.skip(b'#', b';');
                }
                b'%' => {
                    self.nextchar();
                    self.parnum = self.num(self.ch) as i32;
                    self.push(TagType::Param);
                    self.parbal = 1;
                    let mut temp = self.level - 1;
                    while self.parbal != 0 {
                        temp -= 1;
                        match self.stack[temp].tag {
                            TagType::Macro | TagType::Param => self.parbal -= 1,
                            TagType::Loop => {}
                        }
                    }
                    self.chpos = self.stack[temp].pos;
                    self.offset = self.stack[temp].off;
                    loop {
                        self.nextchar();
                        if self.ch == b'#' {
                            self.skip(b'#', b';');
                            self.nextchar();
                        }
                        if self.ch == b',' {
                            self.parnum -= 1;
                        }
                        if self.parnum < 0 || self.ch == b';' {
                            break;
                        }
                    }
                    if self.ch == b';' {
                        self.pop();
                    }
                }
                b',' | b';' => self.pop(),
                _ => {}
            }
            if self.ch == b'$' {
                break;
            }
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut mouse = Mouse::new();

    if args.len() < 2 {
        mouse.load(io::stdin().lock())?;
    } else {
        let file = File::open(&args[1])?;
        mouse.load(file)?;
    }

    mouse.interpret();
    Ok(())
}