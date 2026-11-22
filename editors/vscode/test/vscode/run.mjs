// Starts a real VS Code with the extension loaded and runs suite.cjs inside
// it:  npm run test:vscode   (downloads VS Code into .vscode-test the first time)
import { existsSync, mkdtempSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { downloadAndUnzipVSCode, runTests } from "@vscode/test-electron";

const extensionDevelopmentPath = new URL("../..", import.meta.url).pathname;
const examples = new URL("../../../../examples", import.meta.url).pathname;

// Recent macOS builds name the program "Code", not the "Electron" the test
// tool expects, so look for whatever is there.
let vscodeExecutablePath = await downloadAndUnzipVSCode();
if (!existsSync(vscodeExecutablePath)) {
  const folder = dirname(vscodeExecutablePath);
  vscodeExecutablePath = join(folder, readdirSync(folder)[0]);
}

await runTests({
  vscodeExecutablePath,
  extensionDevelopmentPath,
  extensionTestsPath: new URL("./suite.cjs", import.meta.url).pathname,
  // A short profile path: VS Code's socket path has a length limit.
  launchArgs: [examples, "--disable-extensions", "--disable-workspace-trust", `--user-data-dir=${mkdtempSync(join(tmpdir(), "mv-"))}`],
});
