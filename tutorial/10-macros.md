# Macros

A macro is a named piece of program that can be run from anywhere. Macros do
the job that procedures and functions do in other languages. Grogono's
article calls them macroinstructions.

## Defining and calling a macro

A macro definition starts with `$` and a letter, and ends with `@`. A macro
call is `#`, the letter, and a semicolon.

```mouse
#M;

$M "A MESSAGE!" @
$$
```

```output
A MESSAGE
```

The rules are:

- A macro name is one capital letter, so a program can have up to 26 macros.
  Macro names are separate from variable names: a program can have both a
  variable `M` and a macro `M`.
- There must be no space between `#` and the name in a call, or between `$`
  and the name in a definition.
- Definitions come after the main program and before the final `$$`.
- The main program ends at the first `$`. Usually that is the `$` which starts
  the first definition, so nothing extra needs to be written.

When the interpreter meets `#M;` it jumps to the definition of M, runs it up
to the `@`, and then carries on after the semicolon. A macro can be called as
often as you like:

```mouse
#L; "Mouse!" #L;

$L "==========!" @
$$
```

```output
==========
Mouse
==========
```

## Local variables

Every macro call gets its own complete set of 26 variables. Assigning to a
variable inside a macro does not change the main program's variable of the
same name. This program from the article prints 3, in spite of the
assignment to N inside X:

```mouse
N 3 = #X; N. !

$X N 99 = @
$$
```

```output
3
```

A macro's variables are new each time it is called. They are not set to any
particular value, so give each one a value before using it.

Because a macro cannot see the main program's variables, anything it needs
must be given to it as a parameter. That is the subject of the next chapter.

## Returning early

`@` means "return from this macro". It marks the end of the definition, but
it can also be used earlier, usually inside a condition:

```mouse
#C; "back in the main program!"

$C "one!"
   1 [ "two!" @ ]
   "three!" @
$$
```

```output
one
two
back in the main program
```

"three" is never printed, because the `@` inside the brackets returns first.
This is how a macro chooses between two results, as chapter 11 shows.

## Macros that call macros

A macro definition may contain macro calls.

```mouse
#B;

$B #T; "  |  |!" "  |  |!" #T; @
$T "  +--+!" @
$$
```

```output
  +--+
  |  |
  |  |
  +--+
```

A macro may also call itself. That is called recursion, and it is covered in
the next chapter, because it needs a parameter to be useful.

## Undefined macros

A call to a macro that has not been defined does nothing at all. No error is
reported.

```mouse
"before " #Q; "after!"
$$
```

```output
before after
```

So if a macro seems to do nothing, check that the letter in the call matches
the letter in the definition.

## Summary

| Symbol | Meaning |
|--------|---------|
| `$M` | Start of the definition of macro M (and the end of the main program) |
| `@` | Return from the macro |
| `#M;` | Call macro M |
