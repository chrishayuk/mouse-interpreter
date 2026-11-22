# Mouse in your editor

Two pieces give an editor an understanding of Mouse:

- **`mouse-lsp`**, a language server in [`../lsp`](../lsp). It works with any editor that speaks the Language Server Protocol.
- **A Visual Studio Code extension** in [`vscode`](vscode), which bundles the server and adds syntax highlighting and a run button. Its [README](vscode/README.md) lists the features.

Both are built on [`src/analysis.rs`](../src/analysis.rs), the same code behind `mouse --check`.

## The language server

```sh
cargo install --git https://github.com/chrishayuk/mouse-interpreter mouse-lsp
```

or, in a clone of this repository, `cargo build --release -p mouse-lsp`.

It talks on standard input and output and takes no arguments. It provides diagnostics, hover, go to definition, references, document highlights, rename, completion, signature help, document symbols, folding ranges and semantic tokens. Semantic tokens mean an editor with no Mouse grammar still gets highlighting.

### Neovim (0.11 or later)

```lua
vim.filetype.add({ extension = { mouse = "mouse" } })
vim.lsp.config("mouse", {
  cmd = { "/path/to/mouse-lsp" },
  filetypes = { "mouse" },
})
vim.lsp.enable("mouse")
```

### Helix

In `languages.toml`:

```toml
[language-server.mouse-lsp]
command = "/path/to/mouse-lsp"

[[language]]
name = "mouse"
scope = "source.mouse"
file-types = ["mouse"]
comment-token = "'"
language-servers = ["mouse-lsp"]
```

These two configurations follow each editor's documented format but have not been run here; the server itself is tested over standard input and output by `vscode/test/lsp.test.mjs`.

## Checking a program from the command line

```sh
mouse --check program.mouse
```

prints one line per problem, as `file:line:column: severity: message`, and exits with status 1 if any is an error.
