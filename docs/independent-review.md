# Independent review

## Who reviewed, and what was checked

On 22 November 2026 the repository was reviewed by OpenAI Codex CLI
(codex-cli 0.166.0), run as an independent second model. It is a program,
not a person, and it was not involved in writing the code. It was asked to
check the interpreter, the examples, the tutorial and the README against the
page scans in `docs/scans`, and to run everything the README describes.

It compared the scans with the repository's own copies, ran the programs,
traced the printed Pascal by hand where needed, and ran the build and test
commands. It confirmed that all 27 page images in `docs/scans` match the
originals it was given byte for byte. Its transcription audit covered 22
places where the repository quotes a program from the sources: 10 differed
from the print beyond the usual normalisation (quotes, layout, `^` for the up
arrow), and 12 matched.

Each finding was then checked again by Claude (Anthropic), which made the
changes below, against the scans, by running the programs, and by reading
listing 6. All 10 differences and all 12 matches in the audit were confirmed
against the scans. One further slip, which the review missed, was found in
the process (finding 2).

## Findings and what was done

Commits are on `master`:
`151af9e` (examples and tests), `41b10fe` (documentation),
`ed6d1b1` (rebuilt WebAssembly and playground examples), and the commit that
adds this file.

### 1. Bubble sort fails on one element - accepted

Confirmed: `grogono-bubble-sort.mouse` with input `1 42` stops with
`invalid variable address 0`. The same happens with zero numbers. The fault
is in listing 5 as printed (p. 208), not in the transcription: B's outer FOR
runs "from 1 to 0", which F counts down, so the inner FOR reads element -1.
Listing 6 would index `DATA[0]`, outside its declared bounds `1..260`. It is
a slip in the original.

Done: the listing is kept as printed, and its header describes the slip.
`grogono-bubble-sort-corrected.mouse` adds two marked lines (refuse fewer
than one number; sort only when there are two or more). Tests:
`article_bubble_sort_fails_on_one_number` and
`corrected_bubble_sort_handles_one_number_and_none` in `src/main.rs`; the
corrected example runs one number in `cargo test`. The claim that all five
listings "run correctly" is withdrawn in `tutorial/14-reference.md`.

### 2. The array bound admits a 26th number - accepted, and one more slip found

Confirmed: with `A.25-` a 26th positive number is stored at address 27. The
article's text (p. 204) says "no more than 25 numbers", so this is a slip in
the original; `A.24-` is the minimal correction.

Checking the scan also shows a slip the review did not report: both array
programs on p. 204 read with `BA.+.?=`. The second period fetches the value
of element A instead of keeping its address, so the first number is stored at
address 0, an error here and out of bounds in listing 6. The tutorial had
silently dropped the period.

Done: chapter 9 now shows the printed form, which fails, then the program
without the period, then the printed bound with its 26-number run, then the
corrected bound; every output is checked by `tutorial/check_tutorial.py`.
`examples/array-reverse.mouse` uses both corrections and says so in its
header. Tests: `article_array_bound_lets_a_26th_number_in` and
`article_array_program_has_one_period_too_many`.

### 3. The loader ignores `$` in strings, which listing 6 does not - accepted

Confirmed by reading lines 79-83 of listing 6: `$X` inside a string is
recorded as a definition, and `$$` inside a string ends loading. This
interpreter does neither. It is kept as a reconstruction choice (ours): the
listing's behaviour only matters for strings containing `$`, and Lane's
corrections treat strings as opaque everywhere else.

Done: disclosed in the README and the reference chapter, and the claim that
programs inside the article's limits "behave the same" now names this
exception (and `@` in loops and arithmetic overflow). Test:
`loader_ignores_dollar_signs_inside_strings`.

### 4. Unbalanced brackets are not always reported - accepted

Confirmed: `1 [ $$`, `( $$` and `1 ^ $$` run and exit with status 0. Errors
are found only when execution reaches them, as in listing 6. No code change
was made: rejecting such programs before running would depart from the
original, and `mouse --check` already reports all three.

Done: the README and reference chapter now say exactly which errors the
interpreter reports, and point to `--check` for structure. Test:
`brackets_are_only_checked_when_execution_reaches_them`.

### 5. Transcriptions were adapted, not literal - accepted

All 10 differences were confirmed against the scans. The examples had added
trailing spaces to prompts, `"!"` new lines, a space before `STANDING` and,
in the bubble sort, a new line before `ENTER`.

Done: the `grogono-*` examples now keep the printed strings exactly, with
their outputs regenerated; each header states the only changes (`^`,
straight quotes, spacing outside strings, comments) and gives the page. The
missing space in listing 3 is described as an interpretation and left as
printed. Where the tutorial adapts a program (chapters 9, 11, 13) it now says
how. The test for p. 199 uses the printed `A! T! $$`, and the Lane SKIP test
says how it adapts his fragment.

### 6. "No fixed sizes" is wrong for the control stack - accepted

Done: the README gives the limit of 100,000 nested loops, calls and
parameters, matching the reference chapter.

### 7. Historical framing - accepted

- Comments and error checking are not "beyond the article": p. 220 proposes
  apostrophe comments dropped by the loader, and p. 218 lists the checks an
  implementation should make. Both are now described as going beyond
  listing 6, following the article.
- The tutorial said comments came from "the C version it was ported from".
  Nothing in the repository or its history identifies such a version, so the
  sentence is removed. It can be restored with a reference if one exists.
- The 1983 dialect table is now marked as not checked against a copy of the
  book; only the numbered parameters are attested here (Grogono's 1980
  reply). The dataset's answer about the 1983 version was not changed; it
  repeats the table and is equally unchecked.

### 8. The corrections survive scrutiny, with one omission - accepted

Confirmed against pp. 238 and 240: Lane also corrects line 42,
`CAL := CAL = 1`. The README, the reference chapter and the header of
`src/main.rs` now list it. Every correction is labelled by kind: Lane's five
and Grogono's factorial are slips; `@` inside a loop is a conflict (p. 205
calls `@` a return, line 166 only pops one frame); comments, the string-aware
loader, line breaks in strings, running without `$$`, sizes and error
reports are ours. The space before `STANDING` is no longer added.

### 9. Commands - accepted; the editor checks now pass

The review could not install the editor's npm dependencies (no network).
With them installed, every documented command passes:

| Command | Result |
|---------|--------|
| `cargo build --release` | passed |
| `cargo test --release --workspace` | 56 unit, 2 integration and 22 language-server tests passed |
| `python3 tutorial/check_tutorial.py` | 100 programs checked, 0 failed |
| `python3 tutorial/build_dataset.py` | 80 entries written |
| `node web/build.mjs && node web/test.mjs` | 42 examples checked through WebAssembly, 0 failed |
| `mouse --check examples/*.mouse` | exit 0, the two expected warnings |
| `editors/vscode`: `npm run build`, `npm test` | built; 9 tests passed |
| `editors/vscode`: `npm run test:vscode` | passed |

## Not changed

No interpreter behaviour was changed. Every fix was to programs or text, and
the examples' outputs changed only where a transcription was restored to the
printed strings.
