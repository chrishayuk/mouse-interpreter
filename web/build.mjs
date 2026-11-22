// Builds the files the playground needs:  node web/build.mjs
//   web/mouse.wasm     the interpreter, compiled for WASI
//   web/examples.json  the example programs, with their input
import { execSync } from "node:child_process";
import { copyFileSync, existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";

const root = new URL("..", import.meta.url).pathname;
execSync("cargo build --release --target wasm32-wasip1", { cwd: root, stdio: "inherit" });
copyFileSync(`${root}target/wasm32-wasip1/release/mouse.wasm`, `${root}web/mouse.wasm`);

// Shown first, in this order; the rest follow alphabetically.
const first = ["hello", "arithmetic", "variables", "countdown", "conditions", "fibonacci",
  "factorial", "fizzbuzz", "lane-test", "grogono-hanoi", "grogono-bubble-sort"];

const names = readdirSync(`${root}examples`).filter((f) => f.endsWith(".mouse")).map((f) => f.slice(0, -6));
names.sort((a, b) => {
  const [i, j] = [first.indexOf(a), first.indexOf(b)];
  if (i !== -1 || j !== -1) return (i === -1 ? 99 : i) - (j === -1 ? 99 : j);
  return a.localeCompare(b);
});
const examples = names.map((name) => {
  const input = `${root}examples/${name}.in`;
  return {
    name,
    program: readFileSync(`${root}examples/${name}.mouse`, "utf8"),
    input: existsSync(input) ? readFileSync(input, "utf8") : "",
    endless: name.endsWith("-endless"),
  };
});
writeFileSync(`${root}web/examples.json`, JSON.stringify(examples));
console.log(`web/mouse.wasm and web/examples.json (${examples.length} programs) written`);
