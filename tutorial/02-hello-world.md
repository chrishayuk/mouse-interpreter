# Hello World: printing text

## Printing a string

When the interpreter meets a quote character `"`, it prints every character up
to, but not including, the next quote.

```mouse
"hello world"$$
```

```output
hello world
```

Nothing else is printed: no quotes, and no new line at the end.

## Ending a program

Every Mouse program ends with `$$`. The interpreter stops running at the first
`$` it reaches; the second one tells the loader that there is no more program
to read.

If you type a program at the keyboard and leave off the `$$`, the interpreter
keeps waiting for more of the program. When a program comes from a file, this
interpreter runs it anyway when the file ends, but always write the `$$`:
Grogono's interpreter, and most others, need it.

## Printing a new line

Inside a string, an exclamation mark `!` prints a new line instead of itself.

```mouse
"hello world!"$$
```

```output
hello world
```

This time the output ends with a new line. The `!` can appear anywhere in the
string, and as often as you like:

```mouse
"1!2!3!"$$
```

```output
1
2
3
```

Two in a row leave a blank line:

```mouse
"above!!below!"$$
```

```output
above

below
```

Because `!` always means "new line" and `"` always ends the string, a Mouse
string cannot print an exclamation mark or a quote character. There is no
escape character.

Be careful: outside a string, `!` means something different. It prints a
number, as chapter 5 explains.

## Spaces and line breaks in the program

Outside strings, spaces and line breaks are ignored, so you can lay a program
out however you like. Several strings in a row are simply printed one after
the other:

```mouse
"Hello!"
"World!"
$$
```

```output
Hello
World
```

Inside a string, every space counts and is printed. This interpreter also
prints a line break that is typed inside a string, so the following program
prints two lines:

```mouse
"Hello
World!"$$
```

```output
Hello
World
```

Do not rely on that if you want your programs to run on other Mouse
interpreters. Grogono's original read the program one character at a time in
Pascal, where the end of a line normally arrives as a space. Using `!` works
everywhere.

## Comments

An apostrophe `'` outside a string starts a comment, which runs to the end of
the line. Comments are a feature of this interpreter. The interpreter in the
1979 article has none, though the article suggests adding them in just this
way: an apostrophe, or a pair of braces, that the loader leaves out (p. 220).

```mouse
' This program greets the world.
"hello world!"    ' the ! prints a new line
$$
```

```output
hello world
```

An apostrophe inside a string is printed like any other character:

```mouse
"It's a mouse!"$$
```

```output
It's a mouse
```

## Summary

| Symbol | Meaning |
|--------|---------|
| `"..."` | Print the text between the quotes |
| `!` inside a string | Print a new line |
| `$$` | End of the program |
| `'` | Comment to the end of the line (this interpreter) |
