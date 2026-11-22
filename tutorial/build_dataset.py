#!/usr/bin/env python3
"""Build dataset/mouse.jsonl, a chat-format training set about Mouse.

Every program in an answer is run through the interpreter, and the output
shown in the answer is what it actually printed, so the dataset cannot drift
from the language. Run from this folder after building the interpreter:

    python3 build_dataset.py
"""
import json
import pathlib
import subprocess
import sys

from check_tutorial import interpreter

HERE = pathlib.Path(__file__).resolve().parent
EXAMPLES = HERE.parent / "examples"
SYSTEM = ("You are a knowledgeable assistant that provides detailed information "
          "about the Mouse programming language.")
MOUSE = interpreter()
rows = []


def run(src, stdin=""):
    result = subprocess.run([MOUSE], input=src + "\n" + stdin, capture_output=True,
                            text=True, timeout=10)
    if result.returncode != 0:
        sys.exit(f"program failed:\n{src}\n{result.stderr}")
    return result.stdout.rstrip("\n")


def prog(src, stdin=""):
    """A program, its input if any, and its real output, as markdown."""
    src = src.strip("\n")
    text = f"```mouse\n{src}\n```\n\n"
    if stdin:
        text += f"With the input `{stdin.strip()}` it prints:\n\n"
    else:
        text += "It prints:\n\n"
    return text + f"```\n{run(src, stdin)}\n```"


def qa(question, answer):
    rows.append({"messages": [
        {"role": "system", "content": SYSTEM},
        {"role": "user", "content": question},
        {"role": "assistant", "content": answer.strip()},
    ]})


# ---------------------------------------------------------------- concepts

qa("What is the Mouse programming language?",
   "Mouse is a tiny interpreted programming language designed by Peter Grogono and "
   "published in BYTE magazine in July 1979 in the article \"Mouse: A Language for "
   "Microcomputers\". Every instruction is a single character, calculations are done "
   "on a stack using postfix notation, and there are 26 variables named A to Z. "
   "Despite its size it has conditions, loops, arrays, and macros with parameters "
   "and recursion. Here is a complete program:\n\n" + prog('"Hello, world!"\n$$'))

qa("How do I print Hello World in Mouse?",
   "Put the text between quote characters. An exclamation mark inside the quotes "
   "prints a new line, and `$$` ends the program.\n\n" + prog('"hello world!"$$'))

qa("How do I end a Mouse program?",
   "Every Mouse program ends with `$$`. The interpreter stops running at the first "
   "`$` it reaches, and the second `$` tells the loader that the program text is "
   "complete. Macro definitions, if there are any, go between the main program and "
   "the final `$$`.\n\n" + prog('"Welcome to Mouse!"$$'))

qa("How do I print a new line in Mouse?",
   "Inside a quoted string, an exclamation mark `!` prints a new line instead of "
   "itself.\n\n" + prog('"1!2!3!"$$') +
   "\n\nOutside a string `!` has a different meaning: it prints the number on top of "
   "the stack.")

qa("How do I print several lines of text in Mouse?",
   "Write one string for each line of output and end each with `!`. Spaces and line "
   "breaks between strings are ignored, so the program can be laid out to look like "
   "its output.\n\n" + prog('"Hello!"\n"World!"\n$$'))

qa("Why is there an extra space at the start of my lines in Mouse?",
   "Everything inside the quotes is printed, including spaces. If you put a space "
   "after the `!` that ends a line, that space is printed at the start of the next "
   "line.\n\n" + prog('"first! second!"$$') +
   "\n\nRemove the space after the `!`, or close the string and start a new one.")

qa("Can I print an exclamation mark or a quote character in Mouse?",
   "Not in the 1979 language. Inside a string `!` always prints a new line and `\"` "
   "always ends the string, and there is no escape character. Every other character, "
   "including `$`, `#`, `%` and the backslash, is printed as it is.\n\n"
   + prog('"Price: $5 #1 100%!"$$'))

qa("Are spaces significant in Mouse?",
   "Outside strings, spaces and line breaks are ignored, with one exception: a space "
   "is needed between two numbers, because `5 7` without the space would be read as "
   "57. Inside a string every space is printed. There must also be no space between "
   "`#`, `$` or `%` and the letter that follows it.\n\n" + prog('5 7 ! " " ! $$'))

qa("How do I write a comment in Mouse?",
   "The interpreter in Grogono's 1979 article has no comments, though the article "
   "suggests adding them as an apostrophe or a pair of braces that the loader leaves "
   "out. In this interpreter an apostrophe outside a string starts a comment that "
   "runs to the end of the line.\n\n"
   + prog("' Greet the world.\n\"hello world!\"    ' the ! prints a new line\n$$") +
   "\n\nLater versions of Mouse use `~` for comments instead.")

qa("How do I print a number in Mouse?",
   "Push the number onto the stack by writing it, then use `!` outside a string to "
   "pop it and print it.\n\n" + prog('365 ! $$') +
   "\n\nNumbers are printed with no spaces around them, so print a space yourself if "
   "you want numbers separated: `5 ! \" \" 7 !`.")

qa("Why does 5 7 ! ! print 75 in Mouse and not 5 7?",
   "Two reasons. The stack is last in, first out, so the 7, which was pushed last, "
   "is printed first. And Mouse prints numbers with no spaces around them, so the "
   "two numbers run together. Print a space between them:\n\n"
   + prog('5 7 ! " " ! $$'))

qa("How do I read input in Mouse?",
   "`?` reads a whole number from the keyboard or input and pushes it onto the "
   "stack.\n\n" + prog('"Number: " ? 2 * ! "!"$$', "21\n") +
   "\n\nTo keep the number, store it in a variable: `N ? =`.")

qa("How does arithmetic work in Mouse?",
   "Mouse uses postfix notation: write the operands first and the operator last. The "
   "operators are `+`, `-`, `*` and `/`. Each pops two values and pushes the result. "
   "Division discards the remainder. Operands are in the usual order, so `10 4 -` "
   "is 10 minus 4.\n\n"
   + prog('3 4 + ! " " 10 4 - ! " " 6 7 * ! " " 17 5 / ! $$'))

qa("How do I write (2 + 3) * 4 in Mouse?",
   "Postfix notation needs no brackets. Keep the operands in the same order and "
   "write each operator as soon as both of its operands are complete. So "
   "(2 + 3) * 4 is `2 3 + 4 *`, while 2 + 3 * 4 is `2 3 4 * +`.\n\n"
   + prog('2 3 + 4 * ! " " 2 3 4 * + ! $$'))

qa("How do I write a negative number in Mouse?",
   "Every operator needs two operands, so there is no unary minus. Subtract from "
   "zero: minus seven is `0 7 -`.\n\n" + prog('0 7 - ! $$'))

qa("How do I find a remainder in Mouse?",
   "The 1979 language has no remainder operator. Division discards the remainder, "
   "so N mod D is `N. N. D. / D. * -`. It is convenient as a macro:\n\n"
   + prog('#M,17,5; !\n\n$M %A %A %B / %B * - @\n$$'))

qa("How do variables work in Mouse?",
   "There are 26 variables, A to Z. Writing a letter pushes the variable's "
   "*address*, not its value. The period `.` replaces an address with the value "
   "stored there, and `=` stores a value: write the address, then the value, then "
   "`=`.\n\n" + prog('X 17 =\nY X. 2 * =\n"X = " X. ! ", Y = " Y. ! "!"\n$$'))

qa("How do I assign a value to a variable in Mouse?",
   "Write the variable's letter, then the value, then `=`. The letter pushes the "
   "address, and `=` pops the value and the address and stores one at the other. "
   "`X 17 =` means X := 17.\n\n" + prog('X 17 = X. ! $$'))

qa("What does the period do in Mouse?",
   "The period `.` fetches a value. It pops an address from the stack and pushes the "
   "value stored at that address. `X` is the address of X; `X.` is the value of X. "
   "Forgetting the period is the most common mistake in Mouse: no error is reported, "
   "but the address is used in place of the value.\n\n"
   + prog('X 17 =\n"right: " X. 1 + ! "!"\n"wrong: " X 1 + ! "!"\n$$') +
   "\n\nX is the 24th letter, so its address is 24.")

qa("How do I add one to a variable in Mouse?",
   "Write the address, then the value plus one, then `=`: `N N. 1 + =` is "
   "N := N + 1.\n\n" + prog('N 5 =\nN N. 1 + =\nN. ! $$'))

qa("What does A! print in Mouse?",
   "It prints the address of A, which is 1. A letter pushes an address, and the "
   "addresses of A to Z in the main program are 1 to 26. To print the value of A "
   "write `A. !`.\n\n" + prog('A ! " " Z ! $$'))

qa("How do I swap two variables in Mouse?",
   "Grogono's article uses the stack instead of a temporary variable: push the "
   "address of X, the value of Y, the address of Y and the value of X, then store "
   "twice.\n\n"
   + prog('X 1 = Y 2 =\nX Y. Y X. = =\n"X = " X. ! ", Y = " Y. ! "!"\n$$'))

qa("How do I write an if statement in Mouse?",
   "Write a value followed by a clause in square brackets. The value is popped; the "
   "clause runs if it is greater than zero and is skipped if it is zero or "
   "negative.\n\n" + prog('X 5 =\nX. [ "X is positive!" ]\n"done!"\n$$'))

qa("How do I compare two numbers in Mouse?",
   "The only test is \"greater than zero\", so subtract. X > Y is `X. Y. - [ ... ]`, "
   "X < Y is `Y. X. - [ ... ]`, X >= Y is `X. Y. - 1 + [ ... ]` and X <= Y is "
   "`Y. X. - 1 + [ ... ]`.\n\n"
   + prog('X 3 = Y 8 =\nX. Y. - [ "X is bigger!" ]\nY. X. - [ "Y is bigger!" ]\n$$'))

qa("How do I test whether a number is zero in Mouse?",
   "A number is zero when it is greater than -1 and less than 1. Nest two "
   "conditions so that the inner clause runs only when both are true:\n\n"
   + prog('X 0 =\n1 X. + [ 1 X. - [ "zero!" ] ]\n$$') +
   "\n\nTo test whether two numbers are equal, apply the same test to their "
   "difference.")

qa("Does Mouse have an else?",
   "Not in the 1979 language. Write a second condition with the opposite test. If "
   "the test is expensive, or the clause changes the variables being tested, store "
   "the result first.\n\n"
   + prog('N 7 =\nR N. N. 2 / 2 * - =\nR. [ "odd!" ]\n1 R. - [ "even!" ]\n$$') +
   "\n\nInside a macro you can return with `@` from inside the brackets, and whatever "
   "follows the brackets is the else part.")

qa("How do I write a loop in Mouse?",
   "Put the loop in round brackets. Inside it, `^` pops a value and leaves the loop "
   "if the value is zero or negative. With the test at the start it is a while "
   "loop.\n\n" + prog('N 5 =\n( N. ^\n  N. ! " "\n  N N. 1 - = )\n"!"\n$$'))

qa("How do I count from 1 to 10 in Mouse?",
   "Loop while I is at most 10. `11 I. -` is positive exactly when I <= 10.\n\n"
   + prog('I 1 =\n( 11 I. - ^\n  I. ! " "\n  I I. 1 + = )\n"!"\n$$'))

qa("How do I break out of a loop early in Mouse?",
   "`^` can appear more than once in a loop, and `0 ^` always exits. Put it inside a "
   "condition to leave the loop when something happens.\n\n"
   + prog('N 1 =\n( 11 N. - ^\n  N. ! " "\n  N. 5 - [ "stop!" 0 ^ ]\n  N N. 1 + = )\n$$'))

qa("What is the difference between a while loop and a repeat loop in Mouse?",
   "Only the position of the exit test. `( E ^ C )` tests first, so the body may not "
   "run at all: a while loop. `( C E ^ )` tests last, so the body always runs at "
   "least once: a repeat loop. The test can also go in the middle.\n\n"
   + prog('N 0 =\n( N N. 1 + =\n  N. ! " "\n  5 N. - ^ )\n"!"\n$$'))

qa("Why does my Mouse loop never stop?",
   "A loop only ends when `^` pops a value that is zero or negative. The usual "
   "causes of an endless loop are: no `^` in the loop; forgetting the statement that "
   "changes the loop variable; and forgetting a period, so the test uses the "
   "variable's address, which is always positive. For example `( N ^ ... )` never "
   "ends, while `( N. ^ ... )` ends when N reaches zero.")

qa("How do arrays work in Mouse?",
   "There is no array type. A variable name is an address, and addresses are "
   "numbers, so `A I. +` is the address of element I of an array that starts at A, "
   "and `A I. + .` is its value. Element 0 is A itself, element 1 is B, and so "
   "on.\n\n"
   + prog('I 0 =\n( 5 I. - ^\n  K I. + I. I. * =\n  I I. 1 + = )\n'
          'I 0 =\n( 5 I. - ^\n  K I. + . ! " "\n  I I. 1 + = )\n"!"\n$$') +
   "\n\nThe array shares storage with the ordinary variables, so do not use the "
   "letters it covers for anything else.")

qa("How do I define and call a macro in Mouse?",
   "A definition starts with `$` and a letter and ends with `@`. A call is `#`, the "
   "letter and a semicolon. Definitions come after the main program, which ends at "
   "the first `$`.\n\n" + prog('#M; #M;\n\n$M "A MESSAGE!" @\n$$'))

qa("Do Mouse macros have local variables?",
   "Yes. Every macro call gets its own complete set of 26 variables, so assigning to "
   "a variable inside a macro does not change the main program's variable of the "
   "same name. This program from Grogono's article prints 3:\n\n"
   + prog('N 3 = #X; N. !\n\n$X N 99 = @\n$$'))

qa("How do I pass parameters to a macro in Mouse?",
   "In the call, each actual parameter follows a comma, and the call ends with a "
   "semicolon. In the definition, `%A` is the first parameter, `%B` the second, and "
   "so on.\n\n" + prog('#S,10,3; !\n\n$S %A %B - @\n$$'))

qa("How do I return a value from a macro in Mouse?",
   "Leave it on the stack. A macro that leaves a value on the stack acts as a "
   "function, and its call can be used wherever a number could be.\n\n"
   + prog('#A,2,3; ! " " #A,10,#A,20,30;; !\n\n$A %A %B + @\n$$') +
   "\n\nA macro can also return a result through a parameter: pass the address of a "
   "variable and store into it.")

qa("What is the difference between call by value and call by reference in Mouse?",
   "It depends on whether you pass a value or an address. `#P,A.;` passes the value "
   "of A; `#P,A;` passes its address, which lets the macro store into A.\n\n"
   + prog('N 5 =\n#I,N; #I,N;\nN. !\n\n$I %A %A. 1 + = @\n$$') +
   "\n\nInside the macro `%A` is the address and `%A.` is the value stored there.")

qa("What does @ do in Mouse?",
   "`@` returns from a macro. It ends every macro definition, and it can also be "
   "used earlier, usually inside a condition, to return early.\n\n"
   + prog('"larger: " #X,12,47; ! "!"\n\n$X %A %B - [ %A @ ] %B @\n$$'))

qa("Does Mouse support recursion?",
   "Yes. A macro may call itself, and each call has its own variables and "
   "parameters. This is the recursive factorial from Grogono's article:\n\n"
   + prog('#F,5; !\n\n$F %A [ %A #F,%A 1 -; * @ ] 1 @\n$$'))

qa("Why is my Mouse macro parameter evaluated more than once?",
   "Because parameters are passed as text, not as values. An actual parameter is not "
   "worked out when the macro is called. Each time the formal parameter appears in "
   "the definition, the interpreter runs the text of the actual parameter again, in "
   "the caller's context.\n\n" + prog('#T,"hello ";\n\n$T %A %A %A @\n$$') +
   "\n\nIf a parameter is expensive or has side effects, copy it into a local "
   "variable at the start of the macro: `N %A =`.")

qa("Can I pass a string to a Mouse macro?",
   "Yes. A string is a clause that prints something, so passing a string means "
   "\"print this when the parameter is reached\". Commas and semicolons inside the "
   "string belong to the string.\n\n"
   + prog('#G,"Ada"; #G,"Grace, and Alan";\n\n$G "Hello, " %A ".!" @\n$$'))

qa("What happens if I call a Mouse macro with too few parameters?",
   "A missing parameter is empty: it does nothing. Extra parameters are ignored, and "
   "a call to a macro that has not been defined does nothing at all. None of these "
   "is reported as an error. This example is from Grogono's article:\n\n"
   + prog('#T; #T,"A"; #T,"A","B"; #T,"A","B","C"; #U;\n\n$T "<" %A %B ">!" @\n$$'))

qa("How do I write a for loop in Mouse?",
   "Either write a loop with a counter, or use the FOR macro from Grogono's article. "
   "`#F,I,1,10,body;` runs the body with I going from 1 to 10, counting down "
   "instead if the start is larger than the end.\n\n"
   + prog('#F,I,1,10, I. ! " "; "!"\n\n'
          '$F %A %B = %C %A. - 1 +\n'
          '   [ ( %C %A. - 1 + ^ %D %A %A. 1 + = ) @ ]\n'
          '     ( %A. %C - 1 + ^ %D %A %A. 1 - = ) @\n$$'))

qa("What are all the symbols in the Mouse language?",
   "In the 1979 language: digits push a number; `A`-`Z` push a variable's address; "
   "`?` reads a number; `!` prints a number; `\"...\"` prints text, with `!` inside "
   "meaning a new line; `+ - * /` do arithmetic; `.` fetches the value at an "
   "address; `=` stores a value at an address; `[ ]` is a condition that runs when "
   "the popped value is greater than zero; `( )` is a loop and `^` leaves it when "
   "the popped value is zero or negative; `#X,p,q;` calls macro X with parameters; "
   "`$X` starts a macro definition and `@` returns from it; `%A`-`%Z` run the "
   "actual parameters; `$` stops the program and `$$` ends the program text.")

qa("What is the difference between the 1979 and 1983 versions of Mouse?",
   "The 1979 version, from Grogono's BYTE article, uses `=` to store a value, has no "
   "comparison or remainder operators, and names macro parameters `%A`, `%B` and so "
   "on. The version in Grogono's 1983 book changed several symbols: `:` stores a "
   "value, `=`, `<` and `>` are comparisons that push 1 or 0, `\\` gives a "
   "remainder, parameters are written `1%`, `2%`, and there are operators for "
   "reading and printing single characters, `~` comments and `{` `}` tracing. A "
   "program written for one version will generally not run on an interpreter for "
   "the other.")

qa("Were there any bugs in the original Mouse interpreter?",
   "Yes. In BYTE, June 1980, Tom Lane published \"Comment and Correction for "
   "Mouse\", and Grogono accepted all his points. Line 42 of the listing had a "
   "typographical error, and there was a misprint in the parameter search (a minus sign that should have been a plus); skipping a "
   "condition or loop was confused by brackets inside quoted strings; finding an "
   "actual parameter was confused by strings and by two macro calls next to each "
   "other; and a macro called while a parameter was being evaluated shared local "
   "variables with the macro waiting for that parameter. This program is Lane's "
   "test for the last one. A corrected interpreter prints 1; the original printed "
   "33.\n\n" + prog('#A,#B;;\n\n$A Q 1 = %A Q. ! @\n$B Q 33 = @\n$$'))

qa("What does 'stack underflow' mean in Mouse?",
   "An operator needed a value and the stack was empty. For example `3 + !` gives "
   "`+` only one operand. Check that every operator has both operands before it, "
   "that assignments are written address, value, `=`, and that there is not one `!` "
   "too many. Grogono's original interpreter did not check for this at all.")

# ---------------------------------------------------------------- programs

TASKS = {
    "arithmetic": "Show me examples of arithmetic in Mouse.",
    "variables": "Write a Mouse program that demonstrates variables.",
    "countdown": "Write a Mouse program that counts down from 10.",
    "conditions": "Write a Mouse program that classifies a number as positive, negative or zero.",
    "even-odd": "Write a Mouse program that says whether numbers are even or odd.",
    "sum-to-n": "Write a Mouse program that adds up the numbers from 1 to N.",
    "multiplication-table": "Write a Mouse program that prints a multiplication table.",
    "triangle": "Write a Mouse program that prints a triangle of stars.",
    "diamond": "Write a Mouse program that prints a diamond of stars.",
    "fibonacci": "Write a Mouse program that prints Fibonacci numbers.",
    "factorial": "Write a Mouse program that calculates a factorial.",
    "gcd": "Write a Mouse program that finds the greatest common divisor of two numbers.",
    "fizzbuzz": "Write FizzBuzz in Mouse.",
    "collatz": "Write a Mouse program that prints a Collatz sequence.",
    "primes": "Write a Mouse program that prints prime numbers.",
    "perfect-numbers": "Write a Mouse program that finds perfect numbers.",
    "digits": "Write a Mouse program that sums and reverses the digits of a number.",
    "binary": "Write a Mouse program that converts numbers to binary.",
    "power": "Write a Mouse program that raises a number to a power.",
    "square-root": "Write a Mouse program that calculates an integer square root.",
    "max": "Write a Mouse program that finds the largest and smallest of three numbers.",
    "leap-years": "Write a Mouse program that tests for leap years.",
    "temperature": "Write a Mouse program that prints a Fahrenheit to Celsius table.",
    "pascal-triangle": "Write a Mouse program that prints Pascal's triangle.",
    "roman-numerals": "Write a Mouse program that converts numbers to Roman numerals.",
    "change": "Write a Mouse program that makes change with the fewest coins.",
    "array-reverse": "Write a Mouse program that reads numbers into an array and prints them in reverse.",
    "statistics": "Write a Mouse program that prints statistics for a list of numbers.",
    "ackermann": "Write Ackermann's function in Mouse.",
    "grogono-hanoi": "Write the Towers of Hanoi in Mouse.",
    "grogono-bubble-sort-corrected": "Write a bubble sort in Mouse.",
    "grogono-green-bottles": "Write a Mouse program that prints the Green Bottles song.",
    "grogono-squares-triangles": "Show me an example of mutual recursion in Mouse.",
}
for name, question in TASKS.items():
    src = (EXAMPLES / f"{name}.mouse").read_text()
    stdin_file = EXAMPLES / f"{name}.in"
    stdin = stdin_file.read_text() if stdin_file.exists() else ""
    qa(question, prog(src, " ".join(stdin.split())))

out = HERE / "dataset" / "mouse.jsonl"
with out.open("w") as f:
    for row in rows:
        f.write(json.dumps(row, ensure_ascii=False) + "\n")
print(f"wrote {len(rows)} examples to {out}")
