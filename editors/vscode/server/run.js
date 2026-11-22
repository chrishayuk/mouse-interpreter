// Runs a Mouse program:  node run.js program.mouse
// The interpreter reads the program and then the program's own input from
// standard input, so the file is handed over first and the keyboard after.
"use strict";

const fs = require("fs");
const path = require("path");
const { run } = require("./wasi.js");

const file = process.argv[2];
if (!file) {
  console.error("usage: run.js program.mouse");
  process.exit(2);
}
let program;
try {
  program = fs.readFileSync(file, "utf8");
} catch (error) {
  console.error(`mouse: cannot read ${file}: ${error.message}`);
  process.exit(2);
}
// The loader reads up to "$$". Without one it would take the input as well.
if (!program.includes("$$")) program += "\n$$";
process.exitCode = run(path.join(__dirname, "mouse.wasm"), { prefix: Buffer.from(program + "\n") });
