# Printing several lines: a limerick

A limerick has five lines. In Mouse the whole poem can be one string, with a
`!` wherever a line ends:

```mouse
"There once was a coder named Jack!Who wrote code with a mouse, never slack!His programs were neat!And impossible to beat!Until his mouse got stuck in a crack!"$$
```

```output
There once was a coder named Jack
Who wrote code with a mouse, never slack
His programs were neat
And impossible to beat
Until his mouse got stuck in a crack
```

## Watch the spaces

There is no space after each `!` in that program. Everything inside the quotes
is printed, so a space after the `!` would appear at the start of the next
line:

```mouse
"There once was a coder named Jack! Who wrote code with a mouse, never slack!"$$
```

```output
There once was a coder named Jack
 Who wrote code with a mouse, never slack
```

## Making the program readable

One long string is hard to read. Because spaces and line breaks between
strings are ignored, the limerick can be written as five strings, one per
line. The output is exactly the same as the first version.

```mouse
"There once was a coder named Jack!"
"Who wrote code with a mouse, never slack!"
"His programs were neat!"
"And impossible to beat!"
"Until his mouse got stuck in a crack!"
$$
```

```output
There once was a coder named Jack
Who wrote code with a mouse, never slack
His programs were neat
And impossible to beat
Until his mouse got stuck in a crack
```

This is the best way to write text that runs over several lines: one string
for each line of output, each ending with `!`.

## Do not mix the two kinds of line break

This interpreter also prints a line break typed inside a string (see chapter
2). If you end a line with `!` and then also press Enter inside the same
string, you get two line breaks and a blank line in the output:

```mouse
"His programs were neat!
And impossible to beat!"$$
```

```output
His programs were neat

And impossible to beat
```

Use `!` for new lines, and close the string before starting a new line in the
program.
