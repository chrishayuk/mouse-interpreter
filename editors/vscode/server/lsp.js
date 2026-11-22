// Starts the Mouse language server: the Rust server compiled for WASI,
// talking the Language Server Protocol on standard input and output.
"use strict";

const path = require("path");
const { run } = require("./wasi.js");

process.exitCode = run(path.join(__dirname, "mouse-lsp.wasm"));
