#!/usr/bin/env python3
"""Check that every Mouse program in the tutorial prints what the tutorial says.

A ```mouse block that is a complete program (it contains $$) must be followed
by an ```output block, optionally with an ```input block in between. Each
program is run through the interpreter and its output compared.

    python3 check_tutorial.py            check
    python3 check_tutorial.py --write    rewrite the output blocks from real runs

The interpreter is taken from $MOUSE, or from ../target.
"""
import os
import pathlib
import re
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
BLOCK = re.compile(r"```(\w+)\n(.*?)```", re.S)


def interpreter():
    if os.environ.get("MOUSE"):
        return os.environ["MOUSE"]
    for build in ("release", "debug"):
        path = HERE.parent / "target" / build / "mouse-interpreter"
        if path.exists():
            return str(path)
    sys.exit("no interpreter found: run `cargo build` or set $MOUSE")


def run(mouse, program, stdin):
    result = subprocess.run(
        [mouse], input=program + "\n" + stdin, capture_output=True, text=True, timeout=10
    )
    if result.returncode != 0:
        return result.stdout + result.stderr
    return result.stdout


def main():
    write = "--write" in sys.argv
    mouse = interpreter()
    checked = failed = 0
    for path in sorted(HERE.glob("*.md")):
        text = path.read_text()
        blocks = list(BLOCK.finditer(text))
        edits = []
        for i, block in enumerate(blocks):
            if block.group(1) != "mouse" or "$$" not in block.group(2):
                continue
            stdin, j = "", i + 1
            if j < len(blocks) and blocks[j].group(1) == "input":
                stdin, j = blocks[j].group(2), j + 1
            if j >= len(blocks) or blocks[j].group(1) != "output":
                print(f"{path.name}: program has no output block:\n{block.group(2)}")
                failed += 1
                continue
            actual = run(mouse, block.group(2), stdin)
            checked += 1
            if actual.rstrip("\n") != blocks[j].group(2).rstrip("\n"):
                if write:
                    edits.append((blocks[j].start(2), blocks[j].end(2), actual.rstrip("\n") + "\n"))
                else:
                    failed += 1
                    print(f"{path.name}: output differs for:\n{block.group(2)}"
                          f"--- tutorial says\n{blocks[j].group(2)}--- interpreter prints\n{actual}\n")
        for start, end, new in reversed(edits):
            text = text[:start] + new + text[end:]
        if edits:
            path.write_text(text)
            print(f"{path.name}: rewrote {len(edits)} output block(s)")
    print(f"{checked} programs checked, {failed} failed")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
