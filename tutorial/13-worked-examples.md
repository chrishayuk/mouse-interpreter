# Worked examples

These programs are all in the `examples` folder, where each has a
`.out` file showing its output.

## FizzBuzz

Print the numbers 1 to 15, but "Fizz" for multiples of 3, "Buzz" for
multiples of 5 and "FizzBuzz" for multiples of both.

```mouse
N 1 =
( 16 N. - ^
  P 0 =
  #Z,#M,N.,3;; [ "Fizz" P 1 = ]
  #Z,#M,N.,5;; [ "Buzz" P 1 = ]
  1 P. - [ N. ! ]
  "!"
  N N. 1 + = )

$M %A %A %B / %B * - @
$Z %A 1 + [ 1 %A - [ 1 @ ] ] 0 @
$$
```

```output
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
```

P records whether anything has been printed for this number. If it is still 0
after both tests, `1 P. -` is positive and the number itself is printed.
`#Z,#M,N.,3;;` is "is N mod 3 equal to zero", built from the two macros in
chapter 12.

## Greatest common divisor

Euclid's algorithm: replace the pair (A, B) by (B, A mod B) until B is zero.

```mouse
A ? = B ? =
( B. ^
  T B. =
  B #M,A.,B.; =
  A T. = )
A. !

$M %A %A %B / %B * - @
$$
```

```input
1071 462
```

```output
21
```

`B #M,A.,B.; =` is an ordinary assignment: the address of B, then a value
(supplied by the macro call), then `=`.

## Binary numbers

To print a number in binary, first print everything except its last bit, then
print the last bit. "Everything except the last bit" is the number divided by
two, so the macro calls itself:

```mouse
#B,10; " " #B,255;

$B %A [ #B,%A 2 /; %A %A 2 / 2 * - ! ] @
$$
```

```output
1010 11111111
```

When %A reaches zero the condition fails and the recursion stops.

## Roman numerals

```mouse
N 1979 =
#R,N,1000,"M"; #R,N,900,"CM"; #R,N,500,"D"; #R,N,400,"CD";
#R,N,100,"C";  #R,N,90,"XC";  #R,N,50,"L";  #R,N,40,"XL";
#R,N,10,"X";   #R,N,9,"IX";   #R,N,5,"V";   #R,N,4,"IV";
#R,N,1,"I";

$R ( %A. %B - 1 + ^ %C %A %A. %B - = ) @
$$
```

```output
MCMLXXIX
```

R takes an address, a value and a string. While the number is at least %B, it
prints the string and subtracts %B. N is passed by reference so that each call
carries on from where the last one stopped.

## The Towers of Hanoi

This is listing 4 of Grogono's article. To move a pile of disks from one peg
to another, move all but the largest to the spare peg, move the largest, then
move the rest on top of it.

```mouse
"HOW MANY DISKS? " D ? = "!"
D. [ #H,D.,"LEFT","RIGHT","CENTER"; ]

$H %A [ #H,%A 1 -,%B,%D,%C;
        "MOVE " %B " TO " %C "!"
        #H,%A 1 -,%D,%C,%B; ] @
$$
```

```input
2
```

```output
HOW MANY DISKS? 
MOVE LEFT TO CENTER
MOVE LEFT TO RIGHT
MOVE CENTER TO RIGHT
```

The peg names are strings. Each level of the recursion passes its own
parameters on as `%B`, `%C` and `%D`, so when the innermost call finally
reaches `%B`, the interpreter follows the chain of calls all the way back to
the string in the main program. Finding the right call for each `%` is the
hardest thing the interpreter has to do.

## Bubble sort

Listing 5 of the article is its most elaborate program. It reads an array,
prints it, sorts it and prints it again.

```mouse
"BUBBLE SORT!" #P,A;

$P "HOW MANY NUMBERS? " N ? = N. 26 - [ "TOO MANY!" @ ]
   "!ENTER " N. ! " NUMBERS!" N N. 1 - =
   #F,M,0,N., %A M. + ? =;
   "INPUT ARRAY!"  #F,M,0,N., %A M. + . ! " "; "!"
   #B,%A,0,N.;
   "SORTED ARRAY!" #F,M,0,N., %A M. + . ! " "; "!" @

$B #F,I,%B 1 +,%C,
     #F,J,%C,I.,
       %A J. 1 - + .  %A J. + .  -
         [ #E,%A J. +,%A J. 1 - +; ] ; ; @

$F %A %B = %C %A. - 1 +
   [ ( %C %A. - 1 + ^ %D %A %A. 1 + = ) @ ]
     ( %A. %C - 1 + ^ %D %A %A. 1 - = ) @

$E %A %B. %B %A. = = @
$$
```

```input
6
31 4 15 9 26 5
```

```output
BUBBLE SORT
HOW MANY NUMBERS? 
ENTER 6 NUMBERS
INPUT ARRAY
31 4 15 9 26 5 
SORTED ARRAY
4 5 9 15 26 31 
```

Things to notice:

- The main program is just `#P,A;`. It uses no variables itself, so all 26 of
  them are free to be the array, which P reaches through the address it was
  given.
- F is the FOR macro from chapter 12 and E is the exchange macro from
  chapter 11.
- In B, the body of the outer FOR is another FOR, whose body is a condition
  containing a call to E. Each is an actual parameter, which is why the
  definition of B ends with `; ; @`.

## More programs to read

| File | What it shows |
|------|---------------|
| `fibonacci.mouse` | A simple loop with three variables |
| `collatz.mouse` | Saving a test result before changing the variable |
| `primes.mouse` | A loop with two exits inside a macro |
| `power.mouse` | Recursion with a local variable |
| `ackermann.mouse` | Copying parameters into locals before recursing |
| `pascal-triangle.mouse` | Nested loops and a running product |
| `change.mouse` | Call by reference |
| `call-by-name.mouse` | Parameters being run again on each use |
| `statistics.mouse` | Reading a list of numbers |
| `grogono-green-bottles.mouse` | Listing 3: a song from a recursive macro |
| `grogono-squares-triangles.mouse` | Listing 1: two macros that call each other |
