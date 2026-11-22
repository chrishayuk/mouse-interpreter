# Conditions

## The bracket condition

A condition is written as a value followed by a clause in square brackets:

*value* `[` *clause* `]`

The value is taken off the stack. If it is **greater than zero** the clause
is run. If it is zero or negative, the interpreter skips to the matching `]`.

```mouse
X 5 =
X. [ "X is positive!" ]
"done!"
$$
```

```output
X is positive
done
```

```mouse
X 0 =
X. [ "X is positive!" ]
"done!"
$$
```

```output
done
```

## Other comparisons

"Greater than zero" is the only test Mouse has. There are no `<`, `>` or `=`
comparison operators, and `=` is already used for assignment. Every other
test is built by arranging for a value that is positive exactly when the
condition is true.

| Condition | Mouse | Why |
|-----------|-------|-----|
| X > 0 | `X. [ ... ]` | |
| X ≥ 0 | `X. 1 + [ ... ]` | X + 1 > 0 |
| X < 0 | `0 X. - [ ... ]` | 0 − X > 0 |
| X > Y | `X. Y. - [ ... ]` | X − Y > 0 |
| X ≥ Y | `X. Y. - 1 + [ ... ]` | X − Y + 1 > 0 |
| X < Y | `Y. X. - [ ... ]` | Y − X > 0 |
| X ≤ Y | `Y. X. - 1 + [ ... ]` | Y − X + 1 > 0 |

```mouse
"Enter two numbers: " X ? = Y ? = "!"
X. Y. - [ "the first is bigger!" ]
Y. X. - [ "the second is bigger!" ]
$$
```

```input
3 8
```

```output
Enter two numbers: 
the second is bigger
```

## Testing for equality

X equals zero when it is both greater than −1 and less than 1. Two conditions
can be combined by putting one inside the other, so that the inner clause only
runs when both are true:

```mouse
"Enter a number: " X ? = "!"
1 X. + [ 1 X. - [ "zero!" ] ]
$$
```

```input
0
```

```output
Enter a number: 
zero
```

To test whether X equals Y, apply the same test to their difference:

```mouse
X 7 = Y 7 =
D X. Y. - =
1 D. + [ 1 D. - [ "equal!" ] ]
$$
```

```output
equal
```

Chapter 12 wraps this test up in a macro so that it does not have to be
written out each time.

## There is no "else"

Mouse as published in 1979 has no else. Write a second condition with the
opposite test:

```mouse
"Enter a number: " N ? = "!"
N. 17 - [ "more than seventeen!" ]
18 N. - [ "seventeen or less!" ]
$$
```

```input
12
```

```output
Enter a number: 
seventeen or less
```

When the test is expensive, or the clause changes the variables being tested,
work the answer out once and keep it in a variable. Here R is 1 for an odd
number and 0 for an even one, so `R.` and `1 R. -` are opposite tests:

```mouse
N 7 =
R N. N. 2 / 2 * - =
R. [ "odd!" ]
1 R. - [ "even!" ]
$$
```

```output
odd
```

## Nested conditions

A clause can contain anything, including another condition. The interpreter
counts brackets when it skips, so the right `]` is always found, even when a
bracket appears inside a string:

```mouse
0 [ "this ] is skipped" 1 [ "and so is this" ] ]
"finished!"
$$
```

```output
finished
```

## Summary

| Symbol | Meaning |
|--------|---------|
| `[` | Take a value off the stack; if it is zero or negative, skip to the matching `]` |
| `]` | End of a conditional clause |
