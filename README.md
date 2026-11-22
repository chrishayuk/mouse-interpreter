# mouse-interpreter

An interpreter for Mouse, the tiny stack-based language Peter Grogono
published in BYTE in July 1979, written in Rust with no dependencies.

```mouse
"Hello, world!"
$$
```

## Install

You need [Rust](https://rustup.rs). To install the interpreter and the
language server from this repository:

```bash
cargo install --git https://github.com/chrishayuk/mouse-interpreter mouse-interpreter mouse-lsp
```

That puts `mouse` and `mouse-lsp` in `~/.cargo/bin`. Then:

```bash
echo '"Hello, world!" $$' | mouse
```

For Visual Studio Code, download `mouse-language-0.1.3.vsix` from the
[latest release](https://github.com/chrishayuk/mouse-interpreter/releases/latest)
and install it:

```bash
code --install-extension mouse-language-0.1.3.vsix
```

To build the extension yourself instead, see
[`editors/vscode`](editors/vscode/README.md).

Open any `.mouse` file and you get highlighting, hover help, problems as
you type, and a run button. The extension carries its own copy of the
interpreter and language server, so it works without the `cargo install`
above. For Neovim, Helix and other editors see
[`editors/README.md`](editors/README.md).

Or try Mouse with nothing installed, in the
[playground](https://chrishayuk.com/demos/mouse).

## Build and run

To work on the interpreter itself, clone the repository and:

```bash
cargo build --release
target/release/mouse examples/hello.mouse
```

With no file name the program is read from standard input, up to its `$$`.
Anything after the `$$` is the program's own input:

```bash
echo '"n? " ? 2 * ! "!" $$ 21' | target/release/mouse
```

An error stops the program with a message and exit status 1.

## The language

| Symbol | Action |
|--------|--------|
| `0`-`9` | Push a number |
| `A`-`Z` | Push the address of a variable (A is 1, Z is 26) |
| `?` `!` | Read a number; print a number |
| `"..."` | Print text; `!` inside is a new line |
| `+` `-` `*` `/` | Arithmetic on the top two values |
| `.` | Replace an address with the value stored there |
| `=` | Store: *address value* `=` |
| `[` ... `]` | Run the clause if the popped value is greater than zero |
| `(` ... `)` | Loop; `^` leaves it if the popped value is zero or negative |
| `#X,p,q;` | Call macro X with parameters p and q |
| `$X` ... `@` | Define macro X; `@` returns |
| `%A`-`%Z` | Run the first to twenty-sixth actual parameter |
| `$$` | End of program |

A full tutorial is in [`tutorial/`](tutorial/01-introduction.md).

## Fidelity to the original

The interpreter follows the Pascal program printed as listing 6 of
"Mouse: A Language for Microcomputers" (BYTE, July 1979, pp. 198-220) and was
checked against scans of the article (in [`docs/scans`](docs/scans/README.md)).
Where it differs from the listing, the difference is one of four kinds:

- **slip**: the printed text is wrong under any reading of the sources;
- **conflict**: the sources say two things, and the listing follows the other;
- **interpretation**: our reading of what was meant;
- **ours**: a choice made for this reconstruction.

### Corrections from Tom Lane, accepted by Grogono

Tom Lane's "Comment and Correction for Mouse" (BYTE, June 1980, pp. 238 and
240) corrects one typographical error and four faults in the logic, and
Grogono's reply accepts them all. All are slips, and all are applied:

1. Line 42 was printed `CAL := CAL = 1`, a syntax error, for
   `CAL := CAL - 1`. (The calculation stack here is a Rust `Vec`, so there is
   no line to correct; popping simply works.)
2. A parameter frame increments the balance count when searching for the
   call that a `%` belongs to. Line 176 was printed with a minus sign.
3. Skipping a condition, loop or call ignores brackets inside quoted strings.
4. Finding an actual parameter ignores quoted strings and handles adjacent
   macro calls in an earlier parameter.
5. Local variables are allocated from a separate high-water mark, so a macro
   called from inside an actual parameter does not share variables with the
   macro waiting for that parameter. With the printed listing (once line 42
   is mended), Lane's test prints 33; here it prints 1.

Grogono's reply also corrects a program in the text: the factorial function
on p. 205 lacks the `=` after `N N. 1 -` (a slip).

### Other differences from listing 6

| Behaviour | Listing 6 | Here | Kind |
|-----------|-----------|------|------|
| `@` inside a loop in a macro | Pops only the loop frame (line 166) | Returns from the macro, abandoning the loops | conflict: p. 205 says `@` acts like RETURN |
| Comments | None; p. 220 suggests `'` or braces, dropped by the loader | `'` outside a string starts a comment to the end of the line | ours, following p. 220 |
| `$` inside a string while loading | `$X` in a string is taken as a definition, and `$$` in a string ends the program (lines 79-83) | Ignored inside strings | ours |
| Line break inside a string | Up to the Pascal system | Printed as a line break | ours |
| Program with no `$$` | Never finishes loading | Runs when the input ends | ours |
| Sizes | 500 characters of program, 20 stack entries, 260 variables | No limit on program, calculation stack or variables; at most 100,000 nested loops, calls and parameters | ours |
| Errors | Not checked; p. 218 lists checks an implementation should add | Stack underflow, division by zero, bad addresses, a `#` or `%` without a letter, running off the end of the program, and bad input stop the program with a message | ours, following p. 218 |

Running does not check the structure of a program in advance, any more than
listing 6 did: an unclosed `[` or `(` that execution never needs to close
goes unnoticed, as in `1 [ $$`. `mouse --check` (below) finds those.

A program that stays inside the article's limits and avoids comments, line
breaks inside strings, `$` inside strings, `@` inside loops and arithmetic
overflow should behave
the same here as on listing 6 with Lane's corrections.

### The article's own programs

The programs from the article are in `examples/` as printed: the only changes
are `^` for the printed up arrow, straight quotes, spacing outside strings and
comment lines. Two contain slips, left in place and described in the file:

- Listing 5, the bubble sort (p. 208), fails when asked for one number or
  none, because its FOR macro counts down over an empty range and the sort
  then reads address 0. `grogono-bubble-sort-corrected.mouse` adds two lines
  to fix it.
- The array programs on p. 204 read with `BA.+.?=`, one period too many, so
  the first number is stored at address 0; and the "no more than 25 numbers"
  bound `A.25-` admits a 26th. `array-reverse.mouse` and chapter 9 of the
  tutorial correct both and say so.

In Listing 3, `$W "STANDING ON THE WALL"` has no leading space, so the song
prints `BOTTLESSTANDING`; that is presumably unintended (an interpretation),
but it is left as printed.

An independent review of these claims, and what was done about each
finding, is in [`docs/independent-review.md`](docs/independent-review.md).

The later dialect from Grogono's 1983 book (`:` to store, `<` `=` `>`, `\`,
`1%` parameters) is not implemented.

## Examples

Every program in `examples/` has a `.out` file with its expected output, and
a `.in` file if it reads input. `cargo test` runs them all.

| Program | |
|---------|---|
| `ackermann.mouse` | Ackermann's function |
| `arithmetic.mouse` | Postfix arithmetic |
| `array-reverse.mouse` | Any letter can be used as an array (the article's p. 204 program, corrected) |
| `binary.mouse` | Print numbers in binary with a recursive macro |
| `call-by-name.mouse` | Parameters are run again on every use |
| `change.mouse` | Make change with the fewest coins |
| `collatz.mouse` | The Collatz sequence |
| `conditions.mouse` | Conditions: positive, negative, zero and comparisons |
| `countdown.mouse` | A loop with an exit test |
| `diamond.mouse` | A diamond, using a macro that prints its second parameter %A times |
| `digits.mouse` | Take a number apart |
| `even-odd.mouse` | Even or odd, using a remainder built from division |
| `factorial.mouse` | Factorial three ways |
| `fibonacci.mouse` | The first N Fibonacci numbers |
| `fizzbuzz.mouse` | FizzBuzz |
| `for-loop.mouse` | Grogono's FOR macro |
| `gcd.mouse` | Greatest common divisor by Euclid's algorithm |
| `hello.mouse` | Hello, world |
| `leap-years.mouse` | Leap years |
| `locals.mouse` | Every macro call has its own 26 variables |
| `max.mouse` | A macro that leaves a value on the stack is a function |
| `multiplication-table.mouse` | Nested loops |
| `pascal-triangle.mouse` | Pascal's triangle |
| `perfect-numbers.mouse` | Perfect numbers |
| `power.mouse` | Fast exponentiation by squaring |
| `primes.mouse` | Primes up to a limit, by trial division |
| `roman-numerals.mouse` | Roman numerals |
| `square-root.mouse` | Integer square root by Newton's method |
| `statistics.mouse` | Count, total, largest, smallest and mean of a list ending with 0 |
| `sum-to-n.mouse` | Add up the numbers from 1 to N |
| `swap.mouse` | Exchange two variables using the stack, with no temporary |
| `temperature.mouse` | Fahrenheit to Celsius, in whole degrees |
| `triangle.mouse` | A triangle of stars |
| `variables.mouse` | A letter pushes the ADDRESS of a variable |

Programs from Grogono's article, transcribed as printed (see above):

| Program | |
|---------|---|
| `grogono-bubble-sort.mouse` | Listing 5: read, print, sort and print an array (fails for fewer than two numbers, as printed) |
| `grogono-bubble-sort-corrected.mouse` | Listing 5 with that slip corrected |
| `grogono-factorial.mouse` | Factorials in a loop, from the text of the article |
| `grogono-green-bottles.mouse` | Listing 3: a song from a recursive macro |
| `grogono-hanoi.mouse` | Listing 4: the Towers of Hanoi |
| `grogono-parameters.mouse` | Missing and extra parameters, and an undefined macro |
| `grogono-primes-endless.mouse` | Listing 2: primes for ever (not run by the tests) |
| `grogono-squares-triangles.mouse` | Listing 1: two mutually recursive macros |

## In the browser

`web/` is a playground that runs the interpreter in a web page. The
interpreter is compiled from the same source for WASI (`wasm32-wasip1`), and `web/wasi.js`
is a small host that supplies standard input and output, so no other library
is needed. Each run happens in a worker, which is how an endless program can
be stopped.

`web/editor.js` turns the program box into an editor with colours, problems
as you type, hover help and completion. It loads CodeMirror from a CDN, so
that part needs a network connection; without one the plain box still works.
`web/lsp-worker.js` runs the language server in a page the same way: it is
the WASI build of `mouse-lsp`, given one program and a list of questions per
run. The playground at chrishayuk.com uses it for colours, problems, hover
help and completion.

```bash
node web/build.mjs                  # builds web/mouse.wasm and web/examples.json
node web/test.mjs                   # runs every example through the wasm build
python3 -m http.server -d web 8000  # then open http://localhost:8000
```

Add `?example=lane-test` to the address to open a particular example, and
`&run` to run it straight away.

## In your editor

`mouse --check program.mouse` reads a program without running it and reports
what is wrong: unbalanced brackets and loops, calls with no closing `;`,
macros that are called but not defined, text the interpreter would ignore.

```bash
mouse --check examples/*.mouse
```

The same analysis drives a language server, `mouse-lsp` in `lsp/`, with
hover help for every instruction, go to definition, references, rename,
completion and more. `editors/vscode` is a Visual Studio Code extension that
bundles the server and the interpreter as WebAssembly and adds syntax
highlighting and a run button. See [`editors/README.md`](editors/README.md).

The interpreter itself still has no dependencies. The language server is a
separate crate in the same workspace and uses `serde_json`.

## Tutorial and dataset

`tutorial/` is a 14-chapter tutorial, from hello world to macros and
recursion. Every program in it is followed by its output, and
`python3 tutorial/check_tutorial.py` runs them all and compares.

`tutorial/dataset/mouse.jsonl` is a small chat-format training set about
Mouse. `python3 tutorial/build_dataset.py` rebuilds it; every output in an
answer comes from running the program.

## Tests

```bash
cargo test
```

The unit tests cover each operator, the examples in the text of the article,
each of Lane's corrections, the error messages, and the static analysis. The
integration tests run every example program and compare its output, and check
that every example passes `--check`.

```bash
cargo test -p mouse-lsp                  # the language server
cd editors/vscode && npm install && npm run build && npm test
```

## Licence

Apache License 2.0 - see [LICENSE](LICENSE). That covers the code, the
examples and the tutorial. The page images in `docs/scans` are reproduced
for reference and remain the copyright of their publishers.
