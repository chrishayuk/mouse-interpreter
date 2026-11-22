# Reference

## Every symbol

This is the whole language as published in BYTE in July 1979.

| Symbol | Action |
|--------|--------|
| space, new line | Nothing. Needed only to separate two numbers. |
| `0`–`9` | Read a whole number and push it. |
| `A`–`Z` | Push the address of the variable. In the main program A is 1 and Z is 26. |
| `?` | Read a number from the input and push it. |
| `!` | Pop a value and print it, with no spaces around it. |
| `"` | Print the characters up to the next `"`. Inside the string `!` prints a new line. |
| `+` | Pop two values, push their sum. |
| `-` | Pop two values, push the first minus the second. |
| `*` | Pop two values, push their product. |
| `/` | Pop two values, push the first divided by the second, discarding the remainder. |
| `.` | Pop an address, push the value stored there. |
| `=` | Pop a value, pop an address, store the value at the address. |
| `[` | Pop a value. If it is zero or negative, skip to the matching `]`. |
| `]` | Nothing. Marks the end of a condition. |
| `(` | Start a loop. |
| `^` | Pop a value. If it is zero or negative, leave the loop. |
| `)` | Go back to the start of the loop. |
| `#X` | Call macro X. A call to an undefined macro is skipped. |
| `,` | Introduces each actual parameter of a call. |
| `;` | Ends a macro call. |
| `$X` | Start the definition of macro X. |
| `@` | Return from a macro. |
| `%A`–`%Z` | Run the corresponding actual parameter of the current macro call. |
| `$` | Stop the program. |
| `$$` | End of the program text. |

"First" and "second" refer to the order in which the values were pushed, so
`7 2 -` is 5.

## What a program looks like

```
main program
$A  definition of macro A  @
$B  definition of macro B  @
$$
```

A *clause* is any sequence of strings, expressions, conditions, loops and
macro calls. The body of a condition, a loop, a macro definition or an actual
parameter must be a complete clause: a `[` must have its `]`, and a `(` its
`)`, inside the same clause.

## Where this interpreter follows the article

The interpreter was checked against the Pascal program printed as listing 6
of the article. It includes the corrections made by Tom Lane in "Comment and
Correction for Mouse", BYTE, June 1980, which Grogono accepted in his reply.
All of them are slips in the printed listing:

1. Line 42 was printed `CAL := CAL = 1` instead of `CAL := CAL - 1`.
2. When looking for the call that a `%` belongs to, a parameter frame on the
   stack must be *added* to the count, not subtracted. The printed listing had
   a minus sign on line 176.
3. Skipping over a condition, loop or call ignores brackets that are inside
   quoted strings.
4. Finding an actual parameter ignores commas and semicolons inside quoted
   strings, and copes with two macro calls next to each other in an earlier
   parameter.
5. A macro called while an actual parameter is being run gets its own local
   variables instead of sharing those of the macro that is waiting for the
   parameter.

Grogono's reply also corrected one program in the article: the factorial
function on page 205 should read `$F N %A = 1 ( N. ^ N. * N N. 1 - = ) @`.

All five listings from the article are in the examples folder, transcribed
as printed: `^` for the printed up arrow, straight quotes, spacing outside
strings and comment lines are the only changes. Four run as the article
describes. Listing 5, the bubble sort, has a slip: it fails when asked for
fewer than two numbers. It is kept as printed, and
`grogono-bubble-sort-corrected.mouse` fixes it. The array programs on p. 204
have two slips of their own, described in chapter 9.

## Where this interpreter goes further

Each difference is labelled: *conflict* where the article says two things and
listing 6 follows the other, *ours* where it is a choice made for this
interpreter.

| Feature | Article | This interpreter | Kind |
|---------|---------|------------------|------|
| `@` inside a loop in a macro | Described as a return (p. 205), but listing 6 only leaves the loop | Returns from the macro | conflict |
| Comments | None in listing 6; p. 220 suggests `'` or braces, left out by the loader | `'` outside a string starts a comment that runs to the end of the line | ours |
| `$` inside a string | The loader takes `$X` as a definition and `$$` as the end of the program, even inside a string | Ignored inside strings | ours |
| Line break inside a string | Up to the Pascal system; normally a space | Printed as a line break | ours |
| Program without `$$` | Never finishes loading | Runs when the file ends | ours |
| Program size | 500 characters | No limit | ours |
| Depth of loops, calls and parameters | 20 | 100,000 | ours |
| Calculation stack | 20 values | No limit | ours |
| Variables | 260, enough for 9 nested macro calls | No limit | ours |
| Errors | Not checked by listing 6; p. 218 lists checks to add | Reported, with the position in the program | ours |
| Lower-case letters and unknown symbols | Not allowed | Ignored | ours |

Errors are found as the program runs. A bracket that execution never has to
match, as in `1 [ $$`, is not reported; `mouse --check` finds it.

A program that stays inside the article's limits and avoids comments, line
breaks and `$` inside strings, `@` inside loops and arithmetic overflow
should behave the same on both.

## Error messages

An error stops the program. The offset is counted in characters of the
program after comments and unneeded spaces have been removed.

| Message | Usual cause |
|---------|-------------|
| `stack underflow at '+'` | An operator found too few values on the stack. Often a missing operand or a `!` too many. |
| `division by zero` | The second operand of `/` was zero. |
| `invalid variable address` | `.` or `=` was given a number less than 1. Often the address and value of an assignment are the wrong way round. |
| `ran off the end of the program` | A `[`, `(`, `"` or `#` with no matching `]`, `)`, `"` or `;`. |
| `')' with no matching '('` | A `)` outside any loop. |
| `'%' used outside a macro` | A formal parameter in the main program. |
| `'@' outside a macro` | An `@` in the main program, usually because the `$X` that should start a definition is missing. |
| `expected a letter A-Z after '#'` | A space or other character between `#` or `%` and the name. |
| `'?' expected a number` | The input was not a whole number. |
| `'?' reached the end of input` | The program asked for more numbers than were supplied. |
| `control stack overflow` | A macro that calls itself without ever stopping. |

Some mistakes are not errors. A forgotten period uses an address as a value;
a call to an undefined macro does nothing; a missing parameter is empty.

## Later versions of Mouse

Grogono revised the language in his 1983 book, *Mouse: A Language for
Microcomputers* (Petrocelli Books). Programs written for that version, and
for later interpreters based on it, use different symbols. This tutorial and
interpreter do not implement them, but the differences are worth knowing when
reading other people's programs. The table describes the later dialect as
later interpreters use it; it has not been checked against a copy of the
book, and only the change to numbered parameters is attested by the sources
used here (Grogono's 1980 reply).

| | 1979 (this tutorial) | 1983 and later |
|---|---|---|
| Store | `=` | `:` |
| Equal, less, greater | built from subtraction | `=` `<` `>` push 1 or 0 |
| Remainder | built from `/` and `*` | `\` |
| Parameters | `%A`, `%B` | `1%`, `2%` |
| Print a character | not possible | `!'` |
| Read a character | not possible | `?'` |
| Push a character's code | not possible | `'` followed by the character |
| Comment | none | `~` |
| Tracing | none | `{` on, `}` off |

In his 1980 reply Grogono had already said that he wished he had made `%` an
operator with a number in front of it, which is how the later versions work.

## Sources

- Peter Grogono, "Mouse: A Language for Microcomputers", BYTE, July 1979,
  pp. 198–220.
- Tom Lane and Peter Grogono, "Comment and Correction for Mouse", BYTE, June
  1980.
- Peter Grogono, *Mouse: A Language for Microcomputers*, Petrocelli Books,
  1983.

Scans of both BYTE issues are available on the Internet Archive.
