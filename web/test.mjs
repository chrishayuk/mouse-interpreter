// Runs every example through the WebAssembly build and the browser's WASI
// host, and compares the output with the .out file.   node web/test.mjs
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { run, stdinFor } from "./wasi.js";

const root = new URL("..", import.meta.url).pathname;
const module = new WebAssembly.Module(readFileSync(root + "web/mouse.wasm"));
let checked = 0;
let failed = 0;
for (const file of readdirSync(root + "examples").filter((f) => f.endsWith(".mouse")).sort()) {
  const name = file.slice(0, -6);
  if (name.endsWith("-endless")) continue;
  const program = readFileSync(`${root}examples/${file}`, "utf8");
  const inFile = `${root}examples/${name}.in`;
  const input = existsSync(inFile) ? readFileSync(inFile, "utf8") : "";
  const expected = readFileSync(`${root}examples/${name}.out`, "utf8");
  let out = "";
  const decoder = new TextDecoder();
  const code = run(module, stdinFor(program, input), (fd, bytes) => {
    out += decoder.decode(bytes, { stream: true });
  });
  checked++;
  if (code !== 0 || out !== expected) {
    failed++;
    console.log(`FAIL ${name} (exit ${code})`);
  }
}
// An error must come back as a message and a non-zero exit code.
let err = "";
const code = run(module, stdinFor("3 + ! $$", ""), (fd, bytes) => {
  if (fd === 2) err += new TextDecoder().decode(bytes);
});
if (code !== 1 || !err.includes("stack underflow")) {
  failed++;
  console.log(`FAIL error reporting: exit ${code}, ${JSON.stringify(err)}`);
}
console.log(`${checked} examples checked through WebAssembly, ${failed} failed`);
process.exit(failed ? 1 : 0);
