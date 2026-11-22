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
of the article, and it includes the four corrections made by Tom Lane in
"Comment and Correction for Mouse", BYTE, June 1980, which Grogono accepted
in his reply:

1. When looking for the call that a `%` belongs to, a parameter frame on the
   stack must be *added* to the count, not subtracted. The printed listing had
   a minus sign on line 176.
2. Skipping over a condition, loop or call ignores brackets that are inside
   quoted strings.
3. Finding an actual parameter ignores commas and semicolons inside quoted
   strings, and copes with two macro calls next to each other in an earlier
   parameter.
4. A macro called while an actual parameter is being run gets its own local
   variables instead of sharing those of the macro that is waiting for the
   parameter.

Grogono's reply also corrected one program in the article: the factorial
function on page 205 should read `$F N %A = 1 ( N. ^ N. * N N. 1 - = ) @`.

All five listings from the article are in the examples folder and run
correctly. They were lightly adapted when typed in: `^` for the printed up
arrow, comment lines, and some spacing and new lines added to the prompts.

## Where this interpreter goes further

| Feature | Article | This interpreter |
|---------|---------|------------------|
| Comments | None | `'` outside a string starts a comment that runs to the end of the line |
| Line break inside a string | Up to the Pascal system; normally a space | Printed as a line break |
| Program without `$$` | Never finishes loading | Runs when the file ends |
| Program size | 500 characters | No limit |
| Depth of loops, calls and parameters | 20 | 100,000 |
| Calculation stack | 20 values | No limit |
| Variables | 260, enough for 9 nested macro calls | No limit |
| `@` inside a loop in a macro | Goes wrong | Returns from the macro |
| Errors | Not checked | Reported, with the position in the program |
| Lower-case letters and unknown symbols | Not allowed | Ignored |

A program that stays inside the article's limits and avoids comments and line
breaks inside strings will behave the same on both.

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
reading other people's programs.

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
