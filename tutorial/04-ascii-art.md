# ASCII art

Pictures made of characters are a good test of strings, because every space
matters.

## A tic-tac-toe board

```mouse
"   |   |   !"
"---+---+---!"
"   |   |   !"
"---+---+---!"
"   |   |   !"
$$
```

```output
   |   |   
---+---+---
   |   |   
---+---+---
   |   |   
```

Each string is one row of the board and ends with `!`. Writing one row per
line makes the program look like the picture it draws. The same board can be
written as a single string, which is shorter but harder to check:

```mouse
"   |   |   !---+---+---!   |   |   !---+---+---!   |   |   !"$$
```

```output
   |   |   
---+---+---
   |   |   
---+---+---
   |   |   
```

## A house

```mouse
"    /\    !"
"   /  \   !"
"  /    \  !"
" /______\ !"
" |      | !"
" |  __  | !"
" | |  | | !"
" |_|__|_| !"
$$
```

```output
    /\    
   /  \   
  /    \  
 /______\ 
 |      | 
 |  __  | 
 | |  | | 
 |_|__|_| 
```

A backslash `\` has no special meaning in Mouse. Write one backslash to print
one backslash; writing `\\` prints two.

## Characters you cannot draw with

Two characters cannot be printed from a string:

- `"` ends the string.
- `!` prints a new line.

Everything else, including `$`, `'`, `#` and `%`, is printed as it is when it
appears inside quotes.

```mouse
"Price: $5 (that's 100% of it) #1!"$$
```

```output
Price: $5 (that's 100% of it) #1
```

## Drawing with loops

Typing every row works for small pictures. Once you know loops and macros
(chapters 8 and 10) a program can draw a picture of any size. This one prints
a triangle five rows high:

```mouse
R 1 =
( 6 R. - ^
  C R. =
  ( C. ^ "*" C C. 1 - = )
  "!"
  R R. 1 + = )
$$
```

```output
*
**
***
****
*****
```

See `triangle.mouse` and `diamond.mouse` in the examples folder.
