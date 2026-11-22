# Numbers and the stack

## Printing a number

A whole number in a program is pushed onto the *stack*. Outside a string, `!`
takes the top value off the stack and prints it.

```mouse
365 ! $$
```

```output
365
```

Mouse works only with whole numbers. There are no fractions.

## The stack is last in, first out

Two numbers are pushed by writing one after the other with a space between
them. This is the one place where a space is required: without it, `5 7` would
be read as the single number 57.

```mouse
5 7 ! " " ! $$
```

```output
7 5
```

The 7 was pushed last, so it is printed first. Notice the `" "` between the
two `!` operators. Mouse prints a number with no spaces around it, so a space
must be printed separately or the output would be `75`.

## Reading a number

`?` reads a number from the keyboard, or from the input, and pushes it. This
program reads one number and prints it:

```mouse
? ! $$
```

```input
42
```

```output
42
```

Reading three numbers and then printing three gives them back in reverse
order, because of the stack:

```mouse
? ? ? ! " " ! " " ! $$
```

```input
45 46 47
```

```output
47 46 45
```

To print them in the order they were read, print each one as soon as it has
been read:

```mouse
? ! " " ? ! " " ? ! $$
```

```input
45 46 47
```

```output
45 46 47
```

## Arithmetic

The arithmetic operators are `+`, `-`, `*` and `/`. Each takes two values off
the stack and pushes the result. The operator is written *after* its operands.
This is called postfix notation, or reverse Polish notation.

```mouse
3 4 + ! $$
```

```output
7
```

For subtraction and division the operands are in the same order as in ordinary
arithmetic: `10 4 -` means 10 − 4.

```mouse
10 4 - ! " " 17 5 / ! $$
```

```output
6 3
```

Division throws away the remainder, so `17 5 /` is 3.

## Translating ordinary arithmetic

Postfix notation needs no brackets. Two rules turn an ordinary expression
into a Mouse one:

1. The operands appear in the same order in both.
2. Each operator is written as soon as both of its operands are complete.

| Ordinary | Mouse |
|----------|-------|
| 2 + 3 | `2 3 +` |
| 2 + 3 × 4 | `2 3 4 * +` |
| (2 + 3) × 4 | `2 3 + 4 *` |
| (8 + 4) ÷ (5 − 2) | `8 4 + 5 2 - /` |

```mouse
2 3 4 * + ! " "
2 3 + 4 * ! " "
8 4 + 5 2 - / ! $$
```

```output
14 20 4
```

## Negative numbers

Every operator needs two operands, so there is no way to write −7 directly.
Subtract from zero instead:

```mouse
0 7 - ! $$
```

```output
-7
```

`?` does accept a negative number typed as input.

## Mistakes with the stack

If an operator needs a value and the stack is empty, this interpreter stops
with an error:

```mouse
3 + ! $$
```

```output
mouse: error: stack underflow at '+' (program offset 2)
```

Values left on the stack when the program ends are simply discarded.

## Summary

| Symbol | Meaning |
|--------|---------|
| a number | Push it onto the stack |
| `?` | Read a number and push it |
| `!` | Take the top value off the stack and print it |
| `+` `-` `*` `/` | Take two values off, push the result |
