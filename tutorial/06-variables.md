# Variables

Mouse has 26 variables, named `A` to `Z`. The names must be capital letters.

## A letter is an address

A variable name does not push the variable's value. It pushes its *address*:
a number saying where the variable is kept. The address of `A` is 1, of `B` is
2, and so on up to 26 for `Z`.

```mouse
A ! " " B ! " " Z ! $$
```

```output
1 2 26
```

## Storing a value

The assignment operator is `=`. It needs an address and a value on the stack,
in that order, so an assignment is written *address value* `=`.

```mouse
X 17 =
$$
```

```output
```

That program gives X the value 17 and prints nothing. In another language it
would be written `X = 17`.

## Fetching a value

The period `.` replaces an address on the stack with the value stored there.
So `X` is the address of X and `X.` is the value of X.

```mouse
X 17 =
X. !
$$
```

```output
17
```

To copy one variable into another, fetch the value of the first:

```mouse
X 17 =
Y X. =
Y. ! $$
```

```output
17
```

## Calculating with variables

`X. Y. +` is X + Y. The periods say that the *values* are being added.

```mouse
X 17 =
Y X. 2 * =
"X = " X. ! "!"
"Y = " Y. ! "!"
"X + Y = " X. Y. + ! "!"
$$
```

```output
X = 17
Y = 34
X + Y = 51
```

To add one to a variable, write its address, then its value plus one, then
`=`:

```mouse
N 5 =
N N. 1 + =
N. ! $$
```

```output
6
```

`N N. 1 + =` is the Mouse way of writing `N = N + 1`. You will see it in
almost every loop.

## The most common mistake

Grogono called forgetting the period the most common error in Mouse. No error
is reported; the calculation just uses the address instead of the value.

```mouse
X 17 =
"right: " X. 1 + ! "!"
"wrong: " X 1 + ! "!"
$$
```

```output
right: 18
wrong: 25
```

X is the 24th letter, so `X 1 +` is 24 + 1.

## Reading into a variable

`?` pushes the number it reads, so it can supply the value for `=`:

```mouse
"What is your age? " A ? =
"Next year you will be " A. 1 + ! "!"
$$
```

```input
41
```

```output
What is your age? Next year you will be 42
```

## Using the stack as a spare variable

A value can simply be left on the stack until it is needed. This program from
Grogono's article prints 5, not 6:

```mouse
A A 5 = . A 6 = ! $$
```

```output
5
```

Step by step: `A` pushes the address of A. `A 5 =` sets A to 5 and leaves the
first address where it was. `.` turns that address into the value 5. `A 6 =`
changes A to 6, but the 5 is still on the stack, and `!` prints it.

The same idea exchanges two variables without a third one:

```mouse
X 1 = Y 2 =
X Y. Y X. = =
"X = " X. ! ", Y = " Y. ! "!"
$$
```

```output
X = 2, Y = 1
```

`X Y. Y X.` pushes the address of X, the value of Y, the address of Y and the
value of X. The first `=` stores X's old value in Y, and the second stores Y's
old value in X.

## Summary

| Symbol | Meaning |
|--------|---------|
| `A` to `Z` | Push the address of a variable |
| `.` | Replace an address with the value stored there |
| `=` | Store: *address value* `=` |
