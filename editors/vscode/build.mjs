// Builds everything the extension ships:  node build.mjs
//   server/mouse-lsp.wasm  the language server, compiled for WASI
//   server/mouse.wasm      the interpreter, compiled for WASI
//   dist/extension.js      the extension and its one dependency, bundled
import { execSync } from "node:child_process";
import { copyFileSync } from "node:fs";
import { build } from "esbuild";

const here = new URL(".", import.meta.url).pathname;
const root = new URL("../..", import.meta.url).pathname;
const release = `${root}target/wasm32-wasip1/release`;

execSync("cargo build --release --target wasm32-wasip1 -p mouse-interpreter -p mouse-lsp", { cwd: root, stdio: "inherit" });
copyFileSync(`${release}/mouse-lsp.wasm`, `${here}server/mouse-lsp.wasm`);
copyFileSync(`${release}/mouse.wasm`, `${here}server/mouse.wasm`);

await build({
  entryPoints: [`${here}src/extension.js`],
  outfile: `${here}dist/extension.js`,
  bundle: true,
  platform: "node",
  format: "cjs",
  target: "node18",
  external: ["vscode"],
  minify: true,
  logLevel: "warning",
});
console.log("server/mouse-lsp.wasm, server/mouse.wasm and dist/extension.js written");
