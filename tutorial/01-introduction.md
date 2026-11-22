# The Mouse Programming Language

Mouse is a tiny programming language designed by Peter Grogono and published
in BYTE magazine in July 1979 as "Mouse: A Language for Microcomputers". The
whole language fits in an interpreter of about 200 lines, yet it has variables,
arrays, conditions, loops, and macros with parameters and recursion.

Three ideas explain most of what you will see in a Mouse program:

- **Every instruction is a single character.** `!` prints a number, `?` reads
  one, `+` adds, and so on. There are no keywords.
- **Calculations use a stack.** Values are pushed onto a stack and operators
  take them off again, so arithmetic is written with the operator last:
  `3 4 +` instead of `3 + 4`.
- **Names are single letters.** There are 26 variables, `A` to `Z`, and up to
  26 macros, also named `A` to `Z`.

This tutorial teaches the language as Grogono described it in 1979, with the
corrections published in BYTE in June 1980. A later version of the language,
described in Grogono's 1983 book, changed some of the symbols; chapter 14
lists the differences so that you can read programs written for either.

## Running a Mouse program

Build the interpreter with Cargo, from the top folder of this repository:

```bash
cargo build --release
```

The program is then at `target/release/mouse`. Give it the name of
a file containing a Mouse program:

```bash
target/release/mouse examples/hello.mouse
```

With no file name, the interpreter reads the program from the keyboard. Type
the program, ending with `$$`, and press Enter:

```bash
target/release/mouse
"Hello, world!"$$
Hello, world
```

## A first program

```mouse
"Hello, world!"
$$
```

```output
Hello, world
```

The text between the quotes is printed. The `!` inside the quotes prints a new
line, and `$$` marks the end of the program. The next chapter looks at each of
these in turn.

## How the interpreter works

The interpreter starts at the first character of the program and processes
one character at a time until it reaches a `$`, where it stops. That order is
only changed by conditions, loops, macro calls and macro parameters, which
later chapters describe. Nothing is compiled or checked in advance: a mistake
is only found when the interpreter reaches it.

## The examples

The `examples` folder contains over 40 programs, from
"hello world" to a bubble sort, including all five listings from Grogono's
article. Each one has a `.out` file showing what it prints, and a `.in` file
if it needs input. Every program shown in this tutorial has been run, and the
output shown is what the interpreter printed.
