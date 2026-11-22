// Tokenises Mouse with the same TextMate engine VS Code uses and checks the
// scopes the grammar assigns.
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import test from "node:test";
import oniguruma from "vscode-oniguruma";
import textmate from "vscode-textmate";

const require = createRequire(import.meta.url);
const grammarPath = new URL("../syntaxes/mouse.tmLanguage.json", import.meta.url).pathname;

const wasm = readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm"));
await oniguruma.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
const registry = new textmate.Registry({
  onigLib: Promise.resolve({
    createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
    createOnigString: (text) => new oniguruma.OnigString(text),
  }),
  loadGrammar: async () => textmate.parseRawGrammar(readFileSync(grammarPath, "utf8"), grammarPath),
});
const grammar = await registry.loadGrammar("source.mouse");

/** Each non-blank piece of text with its most specific scope. */
function scopes(source) {
  const out = [];
  let state = textmate.INITIAL;
  for (const line of source.split("\n")) {
    const result = grammar.tokenizeLine(line, state);
    for (const token of result.tokens) {
      const text = line.slice(token.startIndex, token.endIndex);
      if (text.trim()) out.push([text, token.scopes.at(-1)]);
    }
    state = result.ruleStack;
  }
  return out;
}

test("instructions, macros, parameters and comments", () => {
  assert.deepEqual(scopes("#M,7,N.; !  ' call\n$M %A %B - @"), [
    ["#", "keyword.operator.call.mouse"],
    ["M", "entity.name.function.mouse"],
    [",", "punctuation.separator.arguments.mouse"],
    ["7", "constant.numeric.integer.mouse"],
    [",", "punctuation.separator.arguments.mouse"],
    ["N", "variable.other.mouse"],
    [".", "keyword.operator.fetch.mouse"],
    [";", "punctuation.terminator.call.mouse"],
    ["!", "support.function.io.mouse"],
    ["'", "punctuation.definition.comment.mouse"],
    [" call", "comment.line.apostrophe.mouse"],
    ["$", "storage.type.function.mouse"],
    ["M", "entity.name.function.mouse"],
    ["%", "punctuation.definition.variable.mouse"],
    ["A", "variable.parameter.mouse"],
    ["%", "punctuation.definition.variable.mouse"],
    ["B", "variable.parameter.mouse"],
    ["-", "keyword.operator.arithmetic.mouse"],
    ["@", "keyword.control.return.mouse"],
  ]);
});

test("strings span lines, and a bang in one is a new line", () => {
  assert.deepEqual(scopes('"It\'s (1) [A]!\nmore" N'), [
    ['"', "punctuation.definition.string.begin.mouse"],
    ["It's (1) [A]", "string.quoted.double.mouse"],
    ["!", "constant.character.escape.newline.mouse"],
    ["more", "string.quoted.double.mouse"],
    ['"', "punctuation.definition.string.end.mouse"],
    ["N", "variable.other.mouse"],
  ]);
});

test("control flow, and nothing after $$ is program", () => {
  assert.deepEqual(scopes("( N. ^ 1 [ X 2 = ] )\n$$\n12 34 ( not code"), [
    ["(", "keyword.control.loop.mouse"],
    ["N", "variable.other.mouse"],
    [".", "keyword.operator.fetch.mouse"],
    ["^", "keyword.control.loop.mouse"],
    ["1", "constant.numeric.integer.mouse"],
    ["[", "keyword.control.conditional.mouse"],
    ["X", "variable.other.mouse"],
    ["2", "constant.numeric.integer.mouse"],
    ["=", "keyword.operator.assignment.mouse"],
    ["]", "keyword.control.conditional.mouse"],
    [")", "keyword.control.loop.mouse"],
    ["$$", "keyword.control.end.mouse"],
    ["12 34 ( not code", "comment.block.after-end.mouse"],
  ]);
});

test("every example tokenises with no text left unscoped except ignored words", () => {
  const examples = new URL("../../../examples/", import.meta.url).pathname;
  for (const file of readdirSync(examples).filter((f) => f.endsWith(".mouse"))) {
    for (const [text, scope] of scopes(readFileSync(examples + file, "utf8"))) {
      assert.notEqual(scope, "invalid.illegal.lone-dollar.mouse", `${file}: ${text}`);
      if (scope === "source.mouse") assert.match(text.trim(), /^[a-z]+$/, `${file}: unscoped "${text}"`);
    }
  }
});
