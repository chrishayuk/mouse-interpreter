# Arrays

Mouse has no separate array type. Because a variable name is just an address,
and addresses are numbers, arithmetic on addresses reaches neighbouring
variables.

## Addresses can be calculated

The address of B is one more than the address of A. So `A 1 +` is the address
of B, and `A 1 + .` is the value of B.

```mouse
A 7 =
B A. =
A. ! " " B. ! " " A 1 + . ! $$
```

```output
7 7 7
```

`A 7 =` sets A to 7 and `B A. =` copies it to B. The value is then printed
three times: as A, as B, and as "the variable one after A".

## A letter as an array

In general `A I. +` is the address of element I of an array that starts at A,
and `A I. + .` is its value. Element 0 is A itself, element 1 is B, and so on.
Any letter can be the start of an array.

| In other languages | Mouse |
|--------------------|-------|
| `a[i]` (the value) | `A I. + .` |
| `a[i] = x` | `A I. + X. =` |
| `a[i] = a[i] + 1` | `A I. + A I. + . 1 + =` |

This program fills five elements with squares and prints them back:

```mouse
I 0 =
( 5 I. - ^
  K I. + I. I. * =
  I I. 1 + = )
I 0 =
( 5 I. - ^
  K I. + . ! " "
  I I. 1 + = )
"!"
$$
```

```output
0 1 4 9 16 
```

The array starts at K, so it uses K, L, M, N and O.

## Do not use the same address twice

An array shares its storage with the ordinary variables. The program above
uses `I` as the counter and K to O as the array; if the array had started at
E it would have run into I after four elements and overwritten the counter.
Decide which letters belong to the array and do not use them for anything
else.

The main program has 26 variables, so an array starting at A can hold at most
26 elements, and only if no other variable is used. Chapter 10 shows how a
macro gets 26 variables of its own, which frees the main program's variables
to be used as one large array.

## Reading into an array

This program is from Grogono's article. It reads numbers into an array until
it is given a negative one, then prints them in reverse order. A is the index
and the array starts at B.

```mouse
A 0 =
( B A. + ? =
  B A. + . 1 + ^
  A A. 1 + = )
( A. ^
  A A. 1 - =
  B A. + . ! " " )
"!"
$$
```

```input
3 1 4 1 5 9 -1
```

```output
9 5 1 4 1 3 
```

The first loop stores each number in element A, leaves the loop if that
number is negative, and otherwise moves A on by one. The second loop steps A
back down, printing as it goes.

The article improves it so that it cannot read more than 25 numbers, which is
all the array can hold. The first loop gains a second exit:

```mouse
A 0 =
( B A. + ? =
  B A. + . 1 + ^
  A A. 1 + =
  A. 25 - [ "ARRAY FULL!" 0 ^ ] )
( A. ^
  A A. 1 - =
  B A. + . ! " " )
"!"
$$
```

```input
10 20 30 -1
```

```output
30 20 10 
```

## Summary

| Mouse | Meaning |
|-------|---------|
| `A I. +` | The address of element I of the array starting at A |
| `A I. + .` | The value of that element |
| `A I. + V. =` | Store V in that element |
