# Idioms: doing without the missing operators

The 1979 language is deliberately minimal. This chapter collects the standard
ways of writing things that have no operator of their own. Several are wrapped
up as macros that you can copy into your own programs.

## Remainder

There is no remainder operator. Since division throws the remainder away,
`N. D. / D. *` is N rounded down to a multiple of D, and the remainder is
what is left over:

`N. N. D. / D. * -`

As a macro:

```mouse
#M,17,5; ! " " #M,20,5; !

$M %A %A %B / %B * - @
$$
```

```output
2 0
```

## Is it zero? Are two numbers equal?

This macro leaves 1 on the stack if its parameter is zero, and 0 otherwise,
using the nested test from chapter 7:

```mouse
#Z,0; ! #Z,5; ! #Z,0 3 -; !

$Z %A 1 + [ 1 %A - [ 1 @ ] ] 0 @
$$
```

```output
100
```

Two numbers are equal if their difference is zero, so `#Z,X. Y. -;` tests
whether X equals Y. Combined with the remainder macro, it tests whether one
number divides another:

```mouse
N 1 =
( 16 N. - ^
  #Z,#M,N.,3;; [ N. ! " " ]
  N N. 1 + = )
"!"

$M %A %A %B / %B * - @
$Z %A 1 + [ 1 %A - [ 1 @ ] ] 0 @
$$
```

```output
3 6 9 12 15 
```

## Not, and, or

When a condition is held as 1 for true and 0 for false:

| Meaning | Mouse | Why |
|---------|-------|-----|
| not P | `1 P. -` | 1 becomes 0 and 0 becomes 1 |
| P and Q | `P. Q. *` | positive only if both are |
| P or Q | `P. Q. +` | positive if either is |

"And" can also be written by nesting the conditions: `P. [ Q. [ ... ] ]`.

```mouse
P 1 = Q 0 =
P. Q. * [ "both!" ]
P. Q. + [ "at least one!" ]
1 Q. - [ "not Q!" ]
$$
```

```output
at least one
not Q
```

## If … else

Store the result of the test, then use it and its opposite:

```mouse
"Number: " N ? = "!"
T N. 100 - =
T. [ "big!" ]
1 T. - [ "small!" ]
$$
```

```input
250
```

```output
Number: 
big
```

`1 T. -` is positive exactly when T is zero or negative, so the two tests are
opposites whatever value T has.

Inside a macro, `@` gives a neater else: return from inside the brackets, and
whatever follows them is the else part.

```mouse
#S,250; #S,7;

$S %A 100 - [ "big!" @ ] "small!" @
$$
```

```output
big
small
```

## Absolute value, larger, smaller

```mouse
#A,0 12 -; ! " " #X,3,9; ! " " #N,3,9; !

$A %A [ %A @ ] 0 %A - @
$X %A %B - [ %A @ ] %B @
$N %B %A - [ %A @ ] %B @
$$
```

```output
12 9 3
```

## A FOR loop

Grogono's article defines a macro F that works like a FOR statement:
`#F,I,1,10,body;` runs the body with I going from 1 to 10. If the start is
larger than the end, it counts down instead. The body is an actual parameter,
so it is run in the caller's context and can use I.

```mouse
#F,I,1,10, I. ! " ";  "!"
#F,I,5,1,  I. ! " ";  "!"

$F %A %B = %C %A. - 1 +
   [ ( %C %A. - 1 + ^ %D %A %A. 1 + = ) @ ]
     ( %A. %C - 1 + ^ %D %A %A. 1 - = ) @
$$
```

```output
1 2 3 4 5 6 7 8 9 10 
5 4 3 2 1 
```

The first parameter is the *address* of the loop variable, so that F can
store into it. The same macro can be nested:

```mouse
#F,I,1,3, #F,J,1,3, I. J. * ! " "; "!";

$F %A %B = %C %A. - 1 +
   [ ( %C %A. - 1 + ^ %D %A %A. 1 + = ) @ ]
     ( %A. %C - 1 + ^ %D %A %A. 1 - = ) @
$$
```

```output
1 2 3 
2 4 6 
3 6 9 
```

## Lining numbers up

Numbers are printed with no padding. To print in columns, print the padding
yourself, one space for each digit the number lacks:

```mouse
N 1 =
( 2000 N. - ^
  1000 N. - [ " " ] 100 N. - [ " " ] 10 N. - [ " " ]
  N. ! "!"
  N N. 7 * = )
$$
```

```output
   1
   7
  49
 343
```

## Changing a caller's variable

Pass the address, without the period, and use `%A.` for its value and `%A`
for its address inside the macro:

```mouse
N 5 =
#I,N; #I,N; #D,N;
N. !

$I %A %A. 1 + = @
$D %A %A. 2 * = @
$$
```

```output
14
```

## Summary of comparisons

| Test | Mouse value that is positive when true |
|------|----------------------------------------|
| X > Y | `X. Y. -` |
| X ≥ Y | `X. Y. - 1 +` |
| X < Y | `Y. X. -` |
| X ≤ Y | `Y. X. - 1 +` |
| X = Y | `#Z,X. Y. -;` |
| X ≠ Y | `1 #Z,X. Y. -; -` |
| X divides Y | `#Z,#M,Y.,X.;;` |
