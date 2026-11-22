// Runs every example through the runner the "Run Program" command uses, and
// compares what it prints with the expected output.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

const runner = new URL("../server/run.js", import.meta.url).pathname;
const examples = new URL("../../../examples/", import.meta.url).pathname;

test("the built-in interpreter runs every example", () => {
  const names = readdirSync(examples).filter((f) => f.endsWith(".mouse") && !f.endsWith("-endless.mouse"));
  assert.ok(names.length >= 40);
  for (const file of names) {
    const base = examples + file.slice(0, -6);
    const input = existsSync(`${base}.in`) ? readFileSync(`${base}.in`) : "";
    const result = spawnSync(process.execPath, [runner, examples + file], { input });
    assert.equal(result.status, 0, `${file}: ${result.stderr}`);
    assert.equal(result.stdout.toString(), readFileSync(`${base}.out`, "utf8"), file);
  }
});

test("a runtime error is reported and the exit code is not zero", () => {
  const file = join(mkdtempSync(join(tmpdir(), "mouse-")), "broken.mouse");
  writeFileSync(file, '"before " 1 0 / !\n$$\n');
  const result = spawnSync(process.execPath, [runner, file]);
  assert.equal(result.status, 1);
  assert.equal(result.stdout.toString(), "before ");
  assert.match(result.stderr.toString(), /division by zero/);
});

test("a program reads its input from standard input, after the program text", () => {
  const file = join(mkdtempSync(join(tmpdir(), "mouse-")), "double.mouse");
  writeFileSync(file, "? 2 * !"); // no "$$": the runner adds one
  const result = spawnSync(process.execPath, [runner, file], { input: "21\n" });
  assert.equal(result.stdout.toString(), "42");
});
