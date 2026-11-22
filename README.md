# mouse-interpreter

An interpreter for Mouse, the tiny stack-based language Peter Grogono
published in BYTE in July 1979, written in Rust with no dependencies.

```mouse
"Hello, world!"
$$
```

## Build and run

```bash
cargo build --release
target/release/mouse-interpreter examples/hello.mouse
```

With no file name the program is read from standard input, up to its `$$`.
Anything after the `$$` is the program's own input:

```bash
echo '"n? " ? 2 * ! "!" $$ 21' | target/release/mouse-interpreter
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

A full tutorial is in `../mouse-tutorial-ai/markdown`.

## Fidelity to the original

The interpreter follows the Pascal program printed as listing 6 of
"Mouse: A Language for Microcomputers" (BYTE, July 1979, pp. 198-220) and was
checked against scans of the article. It includes the four corrections from
Tom Lane's "Comment and Correction for Mouse" (BYTE, June 1980), which
Grogono accepted:

1. A parameter frame increments the balance count when searching for the
   call that a `%` belongs to. Line 176 of the listing was misprinted with a
   minus sign.
2. Skipping a condition, loop or call ignores brackets inside quoted strings.
3. Finding an actual parameter ignores quoted strings and handles adjacent
   macro calls in an earlier parameter.
4. Local variables are allocated from a separate high-water mark, so a macro
   called from inside an actual parameter does not share variables with the
   macro waiting for that parameter.

It goes beyond the article in these ways:

- `'` outside a string starts a comment that runs to the end of the line.
- A line break inside a string is printed as a line break.
- A program with no `$$` runs when its input ends.
- There are no fixed sizes for the program, the stacks or the variables.
- `@` inside a loop returns from the macro.
- Stack underflow, division by zero, bad addresses, unbalanced brackets and
  bad input are reported instead of being left undefined.

The later dialect from Grogono's 1983 book (`:` to store, `<` `=` `>`, `\`,
`1%` parameters) is not implemented.

## Examples

Every program in `examples/` has a `.out` file with its expected output, and
a `.in` file if it reads input. `cargo test` runs them all.

| Program | |
|---------|---|
| `ackermann.mouse` | Ackermann's function |
| `arithmetic.mouse` | Postfix arithmetic |
| `array-reverse.mouse` | Any letter can be used as an array |
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

Programs from Grogono's article, transcribed from the printed listings:

| Program | |
|---------|---|
| `grogono-bubble-sort.mouse` | Listing 5: read, print, sort and print an array |
| `grogono-factorial.mouse` | Factorials in a loop, from the text of the article |
| `grogono-green-bottles.mouse` | Listing 3: a song from a recursive macro |
| `grogono-hanoi.mouse` | Listing 4: the Towers of Hanoi |
| `grogono-parameters.mouse` | Missing and extra parameters, and an undefined macro |
| `grogono-primes-endless.mouse` | Listing 2 as printed: primes for ever (not run by the tests) |
| `grogono-squares-triangles.mouse` | Listing 1: two mutually recursive macros |

## Tests

```bash
cargo test
```

The unit tests cover each operator, the examples in the text of the article,
each of Lane's corrections, and the error messages. The integration test runs
every example program and compares its output.
