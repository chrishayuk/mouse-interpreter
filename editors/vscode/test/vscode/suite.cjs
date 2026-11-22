// Runs inside VS Code. Exercises the extension the way a person would: open
// a program, hover, jump, rename, see problems, and run it.
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const vscode = require("vscode");

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function until(what, probe, seconds = 20) {
  for (let i = 0; i < seconds * 10; i++) {
    const value = await probe();
    if (value) return value;
    await sleep(100);
  }
  throw new Error(`timed out waiting for ${what}`);
}

const hoverText = (hovers) =>
  hovers.flatMap((h) => h.contents).map((c) => (typeof c === "string" ? c : c.value)).join("\n");

exports.run = async function run() {
  const folder = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const document = await vscode.workspace.openTextDocument(path.join(folder, "lane-test.mouse"));
  await vscode.window.showTextDocument(document);
  assert.equal(document.languageId, "mouse", "the .mouse extension selects the language");

  const call = new vscode.Position(4, 1); // the A of "#A,#B;;"
  assert.equal(document.lineAt(4).text, "#A,#B;;");

  // The server is ready when it answers a hover.
  const hovers = await until("a hover from the language server", async () => {
    const found = await vscode.commands.executeCommand("vscode.executeHoverProvider", document.uri, call);
    return found.length ? found : undefined;
  });
  assert.match(hoverText(hovers), /Call of macro `\$A`/);
  assert.match(hoverText(hovers), /\$A Q 1 = %A Q\. ! @/);
  console.log("hover ok");

  const definitions = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", document.uri, call);
  assert.equal(definitions.length, 1);
  assert.equal((definitions[0].range || definitions[0].targetRange).start.line, 6);
  console.log("definition ok");

  const references = await vscode.commands.executeCommand("vscode.executeReferenceProvider", document.uri, call);
  assert.equal(references.length, 2);

  const edit = await vscode.commands.executeCommand("vscode.executeDocumentRenameProvider", document.uri, call, "Z");
  assert.equal(edit.entries()[0][1].length, 2, "renaming a macro changes its call and its definition");
  await assert.rejects(
    vscode.commands.executeCommand("vscode.executeDocumentRenameProvider", document.uri, call, "B"),
    /already a macro \$B/
  );
  console.log("references and rename ok");

  const symbols = await vscode.commands.executeCommand("vscode.executeDocumentSymbolProvider", document.uri);
  assert.deepEqual(symbols.map((s) => s.name), ["$A", "$B"]);

  const legend = await vscode.commands.executeCommand("vscode.provideDocumentSemanticTokensLegend", document.uri);
  assert.ok(legend.tokenTypes.includes("parameter"));
  const tokens = await vscode.commands.executeCommand("vscode.provideDocumentSemanticTokens", document.uri);
  assert.ok(tokens.data.length >= 5 * 20);
  console.log("symbols and semantic tokens ok");

  const completions = await vscode.commands.executeCommand("vscode.executeCompletionItemProvider", document.uri, new vscode.Position(4, 1), "#");
  assert.deepEqual(completions.items.map((i) => i.label).filter((l) => l.length === 1), ["A", "B"]);

  assert.deepEqual(vscode.languages.getDiagnostics(document.uri), [], "Lane's test has no problems");

  // A broken program shows its problems.
  const broken = await vscode.workspace.openTextDocument({ language: "mouse", content: "1 [ 2 !\n#Q;\nx\n$$\n" });
  await vscode.window.showTextDocument(broken);
  const problems = await until("diagnostics", () => {
    const found = vscode.languages.getDiagnostics(broken.uri);
    return found.length ? found : undefined;
  });
  assert.deepEqual(problems.map((p) => p.code), ["unclosed-bracket", "undefined-macro", "ignored-text"]);
  assert.equal(problems[0].severity, vscode.DiagnosticSeverity.Error);
  assert.equal(problems[0].source, "mouse");
  console.log("diagnostics ok");

  // Run Program: a task that ends with exit code 0.
  await vscode.window.showTextDocument(document);
  const finished = new Promise((resolve) => {
    const listener = vscode.tasks.onDidEndTaskProcess((event) => {
      if (event.execution.task.definition.type === "mouse") {
        listener.dispose();
        resolve(event.exitCode);
      }
    });
  });
  await vscode.commands.executeCommand("mouse.run");
  assert.equal(await Promise.race([finished, sleep(20000).then(() => "timed out")]), 0);
  console.log("run ok");
};
