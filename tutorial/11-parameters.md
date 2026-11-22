# Macro parameters

## Passing values to a macro

In a call, the *actual parameters* follow the macro name, each one preceded
by a comma. The call ends with a semicolon.

`#M,X,Y;` calls M with two parameters, `X` and `Y`.

Inside the definition, the *formal parameters* are written `%A` for the
first, `%B` for the second, and so on up to `%Z`. There must be no space
between `%` and the letter. This macro prints its parameter:

```mouse
#P,3; " " #P,4 5 +;

$P %A ! @
$$
```

```output
3 9
```

## By value and by reference

A parameter can be a value or an address, and the macro must be written to
expect one or the other. This is the example from Grogono's article (p. 205),
which prints `351` as `#P,3; A5= #P,A.; #P,A;`. Here A is set first and
`" "` separates the numbers:

```mouse
A 5 =
#P,3; " " #P,A.; " " #P,A;

$P %A ! @
$$
```

```output
3 5 1
```

`#P,3;` prints 3. `#P,A.;` passes the *value* of A and prints 5: a call by
value. `#P,A;` passes the *address* of A and prints 1: a call by reference.

A macro that is given an address can store into the caller's variable, which
is how a macro hands back a result. This one adds its first two parameters
and stores the answer in the third:

```mouse
#A,2,3,X;  #A,3,4,Y;  #A,X.,Y.,Z;
X. ! " " Y. ! " " Z. !

$A %C %A %B + = @
$$
```

```output
5 7 12
```

In the last call the first two parameters are the values `X.` and `Y.`, and
the third is the address `Z`.

The exchange trick from chapter 6 makes a useful macro. Both parameters must
be addresses:

```mouse
X 1 = Y 2 =
#E,X,Y;
"X = " X. ! ", Y = " Y. ! "!"

$E %A %B. %B %A. = = @
$$
```

```output
X = 2, Y = 1
```

## Functions

A macro that leaves a value on the stack acts as a function. The call can be
used anywhere a number could be:

```mouse
#A,2,3; ! " "
#A,10,#A,20,30;; !

$A %A %B + @
$$
```

```output
5 60
```

The second line shows a macro call used as a parameter of another macro call.
Each call needs its own semicolon, which is why there are two together.

Here is factorial as a function. N is a local variable of F:

```mouse
#F,5; #F,6; + !

$F N %A = 1 ( N. ^ N. * N N. 1 - = ) @
$$
```

```output
840
```

5! + 6! is 120 + 720.

## Choosing a result

With `@` inside a condition, a function can return one of two values:

```mouse
"larger:  " #X,12,47; ! "!"
"smaller: " #N,12,47; ! "!"

$X %A %B - [ %A @ ] %B @
$N %B %A - [ %A @ ] %B @
$$
```

```output
larger:  47
smaller: 12
```

In X, if %A − %B is positive the macro pushes %A and returns. Otherwise it
falls through to push %B.

## Recursion

A macro may call itself. This is the recursive factorial from the article:

```mouse
#F,5; !

$F %A [ %A #F,%A 1 -; * @ ] 1 @
$$
```

```output
120
```

If %A is greater than zero, the result is %A times the factorial of %A − 1.
Otherwise it is 1. Each call has its own variables and its own parameters, so
the calls do not interfere with each other.

## Parameters are text, not values

This is the most unusual thing about Mouse, and the most important to
understand. An actual parameter is **not** worked out when the macro is
called. Instead, each time the formal parameter appears in the definition,
the interpreter goes back to the call and runs the text of the actual
parameter there, using the *caller's* variables.

So a parameter that is used three times is run three times:

```mouse
#T,"hello ";

$T %A %A %A @
$$
```

```output
hello hello hello 
```

A parameter of `?` reads a new number on every use:

```mouse
#S,?; !

$S %A %A + @
$$
```

```input
20 22
```

```output
42
```

And a parameter may be any clause at all, even an assignment. The caller's X
goes up by one each time `%A` is reached:

```mouse
X 0 =
#T,X X. 1 + =;
X. !

$T %A %A %A @
$$
```

```output
3
```

Two practical rules follow from this.

**Copy a parameter into a local variable** if you use it more than once and
it is expensive or has side effects. `N %A =` at the start of a macro runs
the parameter once and keeps the result.

**A parameter can be a piece of program.** This makes it possible to write
your own control structures. Here is a macro that repeats its second
parameter as many times as its first says:

```mouse
#R,3,"ho ";  "!"
#R,5,"*";    "!"

$R K %A = ( K. ^ %B K K. 1 - = ) @
$$
```

```output
ho ho ho 
*****
```

## Strings as parameters

A string is a clause that prints something, so passing a string means "print
this when you reach the parameter". Commas and semicolons inside the string
are part of the string, not part of the call.

```mouse
#G,"Ada"; #G,"Grace, and Alan";

$G "Hello, " %A ".!" @
$$
```

```output
Hello, Ada.
Hello, Grace, and Alan.
```

## Missing and extra parameters

If a call supplies fewer parameters than the macro uses, the missing ones are
empty: they do nothing. Extra parameters are ignored. This example is from
the article; the last call is to a macro that does not exist.

```mouse
#T;  #T,"A";  #T,"A","B";  #T,"A","B","C";  #U;

$T "<" %A %B ">!" @
$$
```

```output
<>
<A>
<AB>
<AB>
```

## Passing an array

An array is passed by the address of its first element. This macro from the
article adds up the first %B elements of the array %A and stores the total
in %C:

```mouse
A 10 = B 20 = C 30 =
#S,A,3,Z;
Z. !

$S %C 0 = N 0 =
   ( %B N. - ^
     %C %A N. + . %C. + =
     N N. 1 + = ) @
$$
```

```output
60
```

## Summary

| Symbol | Meaning |
|--------|---------|
| `#M,p,q;` | Call macro M with actual parameters p and q |
| `%A` … `%Z` | Run the first … twenty-sixth actual parameter |
| `,` | Starts each actual parameter |
| `;` | Ends the call |
