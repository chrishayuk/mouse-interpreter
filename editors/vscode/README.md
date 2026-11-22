# Mouse for Visual Studio Code

Editor support for [Mouse](https://github.com/chrishayuk/mouse-interpreter), the programming language Peter Grogono published in BYTE in July 1979. Nearly every instruction in Mouse is a single character, so an editor that can say what each one does helps a great deal.

Nothing else needs installing: the language server and the interpreter are built into the extension as WebAssembly.

## What you get

- **Syntax highlighting** for strings, comments, numbers, variables, macro definitions (`$A`), calls (`#A,…;`) and parameters (`%A`). Text after the closing `$$` is shown as not part of the program.
- **Hover** on any character to see what it does. Hover on a macro call to see its definition and its comment.
- **Problems as you type:** unbalanced `[ ]` and `( )`, a call with no closing `;`, a string that is never closed, `%A` or `@` outside a macro, `^` outside a loop, a call to a macro that is not defined, a loop with no `^`, a macro with no `@`, a missing `$$`, and text the interpreter would silently ignore.
- **Go to definition** and **find all references** for macros.
- **Rename** a macro, or a variable within the main program or one macro. Each macro call has its own 26 variables, so `Q` in `$A` and `Q` in `$B` are renamed separately.
- **Completion:** macros after `#`, parameters after `%`, and templates for a loop, an if and a macro.
- **Signature help** inside a call's arguments, an **outline** of the macros, and **folding** for macros, loops and comment blocks.
- **Run Program:** the play button at the top of the editor, or Cmd+Enter (Ctrl+Enter on Windows and Linux). The program runs in a terminal, so it can ask for input with `?`.

## Settings

| Setting | What it does |
| --- | --- |
| `mouse.server.path` | Use a `mouse-lsp` program you built yourself as the language server. |
| `mouse.interpreter.path` | Run programs with an interpreter you built yourself. |
| `mouse.trace.server` | Log the messages exchanged with the language server. |

## Building it

From this folder, with Rust (and its `wasm32-wasip1` target) and Node installed:

```sh
npm install
npm run build        # compiles the server and interpreter to WebAssembly, bundles the extension
npm test             # grammar, language server and runner tests
npm run test:vscode  # opens a real VS Code and drives the extension
npm run package      # writes mouse-language-0.1.2.vsix
code --install-extension mouse-language-0.1.2.vsix
```

To try it without installing, open this folder in VS Code and press F5.

## How it fits together

| File | What it is |
| --- | --- |
| `syntaxes/mouse.tmLanguage.json` | The highlighting grammar. |
| `language-configuration.json` | Comment character, brackets and auto-closing pairs. |
| `src/extension.js` | Starts the language server and provides Run Program. |
| `server/lsp.js`, `server/mouse-lsp.wasm` | The language server from `../../lsp`, compiled for WASI. |
| `server/run.js`, `server/mouse.wasm` | The interpreter, compiled for WASI. |
| `server/wasi.js` | A small WASI host for Node, with no dependencies. |
