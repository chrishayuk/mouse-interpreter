# Loops

## The loop and its exit

A loop is written in round brackets. The clause between them is repeated for
ever, unless the exit operator `^` stops it:

`(` *clause* *value* `^` *clause* `)`

`^` takes a value off the stack. If the value is **zero or negative**, the
loop ends and the program carries on after the `)`. Otherwise the loop
continues. So a loop runs for as long as its exit value is greater than zero.

(In the 1979 article the exit operator is printed as an upward arrow `↑`,
which is how the `^` character looked on terminals of the time.)

```mouse
N 5 =
( N. ^
  N. ! " "
  N N. 1 - = )
"!"
$$
```

```output
5 4 3 2 1 
```

Read this as: while N is greater than zero, print N and take one away.

## Where the exit goes

The exit test can go anywhere in the loop. At the start, it makes a "while"
loop, which may not run at all:

`( E ^ C )` — while E > 0 do C

At the end, it makes a "repeat" loop, which always runs at least once:

`( C E ^ )` — repeat C until E ≤ 0

```mouse
N 0 =
( N N. 1 + =
  N. ! " "
  5 N. - ^ )
"!"
$$
```

```output
1 2 3 4 5 
```

In the middle, part of the loop runs one more time than the rest. This prints
commas *between* the numbers but not after the last one:

```mouse
N 1 =
( N. !
  5 N. - ^
  ", "
  N N. 1 + = )
"!"
$$
```

```output
1, 2, 3, 4, 5
```

## Counting up

To count from 1 to 10, the loop must continue while I ≤ 10. From the table in
chapter 7, that is `10 I. - 1 +`, or more simply `11 I. -`.

```mouse
I 1 =
( 11 I. - ^
  I. ! " "
  I I. 1 + = )
"!"
$$
```

```output
1 2 3 4 5 6 7 8 9 10 
```

## Adding things up

```mouse
"N: " N ? = "!"
S 0 = I 1 =
( N. I. - 1 + ^
  S S. I. + =
  I I. 1 + = )
"1 + 2 + ... + " N. ! " = " S. ! "!"
$$
```

```input
100
```

```output
N: 
1 + 2 + ... + 100 = 5050
```

## Factorial

This program from Grogono's article reads N and prints N factorial,
1 × 2 × 3 × … × N.

```mouse
N ? = F 1 =
( N. ^ F F. N. * = N N. 1 - = )
F. ! $$
```

```input
5
```

```output
120
```

A shorter version keeps the running product on the stack instead of in F.
The `1` pushes the starting value, each `N. *` multiplies it by N, and the
final `!` prints it.

```mouse
N ? = 1
( N. ^ N. * N N. 1 - = )
! $$
```

```input
5
```

```output
120
```

## More than one exit

A loop can have several exits. Inside a condition, `0 ^` always leaves the
loop, which gives the effect of a "break". This loop stops at 6 because of the
second exit, although the first would have let it run to 10:

```mouse
N 1 =
( 11 N. - ^
  N. ! " "
  N. 5 - [ "stop!" 0 ^ ]
  N N. 1 + = )
$$
```

```output
1 2 3 4 5 6 stop
```

## Loops inside loops

Loops can be nested. Each `^` leaves the loop it is written in.

```mouse
I 1 =
( 4 I. - ^
  J 1 =
  ( 4 J. - ^
    I. J. * ! " "
    J J. 1 + = )
  "!"
  I I. 1 + = )
$$
```

```output
1 2 3 
2 4 6 
3 6 9 
```

A loop that keeps asking for input until it is given zero is a common
pattern. Here the outer loop does the asking and the inner loop does the
work:

```mouse
( "ENTER A NUMBER " M ? = M. ^
  N M. = 1 ( N. ^ N. * N N. 1 - = )
  "FACTORIAL (" M. ! ") = " ! "!" )
$$
```

```input
3 7 0
```

```output
ENTER A NUMBER FACTORIAL (3) = 6
ENTER A NUMBER FACTORIAL (7) = 5040
ENTER A NUMBER 
```

## Loops that never end

A loop with no `^`, or whose exit value never drops to zero, runs for ever.
Stop the interpreter with Ctrl-C. The usual causes are forgetting the line
that changes the loop variable, and forgetting a period so that the test uses
an address, which is always positive.

## Summary

| Symbol | Meaning |
|--------|---------|
| `(` | Start of a loop |
| `^` | Take a value off the stack; if it is zero or negative, leave the loop |
| `)` | Go back to the start of the loop |
