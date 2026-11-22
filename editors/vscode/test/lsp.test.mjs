// Talks to the language server the way an editor does: over standard input
// and output. It tests the WebAssembly build the extension ships and, when
// it has been built, the native program other editors would use.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import test from "node:test";

const here = new URL("..", import.meta.url).pathname;
const native = new URL("../../../target/release/mouse-lsp", import.meta.url).pathname;
const servers = [["WebAssembly", process.execPath, [`${here}server/lsp.js`, "--stdio"]]];
if (existsSync(native)) servers.push(["native", native, []]);

function connect(command, args) {
  const child = spawn(command, args, { stdio: ["pipe", "pipe", "inherit"] });
  let buffer = Buffer.alloc(0);
  const waiting = [];
  const arrived = [];
  child.stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      const header = buffer.indexOf("\r\n\r\n");
      if (header === -1) return;
      const length = Number(/Content-Length: (\d+)/i.exec(buffer.subarray(0, header).toString())[1]);
      if (buffer.length < header + 4 + length) return;
      const message = JSON.parse(buffer.subarray(header + 4, header + 4 + length).toString());
      buffer = buffer.subarray(header + 4 + length);
      const next = waiting.shift();
      if (next) next(message);
      else arrived.push(message);
    }
  });
  const send = (message) => {
    const body = JSON.stringify({ jsonrpc: "2.0", ...message });
    child.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
  };
  const receive = () => new Promise((resolve) => (arrived.length ? resolve(arrived.shift()) : waiting.push(resolve)));
  let id = 0;
  const request = async (method, params) => {
    send({ id: ++id, method, params });
    const reply = await receive();
    assert.equal(reply.id, id);
    return reply.result;
  };
  const exited = new Promise((resolve) => child.on("exit", resolve));
  return { send, receive, request, exited };
}

const uri = "file:///lane.mouse";
const program = `' Lane's test.
#A,#B;;

$A Q 1 = %A Q. ! @   ' prints its own Q
$B Q 33 = @
$$
`;

for (const [name, command, args] of servers) {
  test(`${name} server: a session from initialize to exit`, async () => {
    const server = connect(command, args);
    const initialised = await server.request("initialize", { processId: null, rootUri: null, capabilities: {} });
    assert.equal(initialised.serverInfo.name, "mouse-lsp");
    assert.equal(initialised.capabilities.hoverProvider, true);
    server.send({ method: "initialized", params: {} });

    server.send({ method: "textDocument/didOpen", params: { textDocument: { uri, languageId: "mouse", version: 1, text: program } } });
    const clean = await server.receive();
    assert.equal(clean.method, "textDocument/publishDiagnostics");
    assert.deepEqual(clean.params.diagnostics, []);

    const hover = await server.request("textDocument/hover", { textDocument: { uri }, position: { line: 1, character: 1 } });
    assert.match(hover.contents.value, /Call of macro `\$A`/);
    assert.match(hover.contents.value, /prints its own Q/);

    const definition = await server.request("textDocument/definition", { textDocument: { uri }, position: { line: 1, character: 4 } });
    assert.deepEqual(definition.range.start, { line: 4, character: 0 });

    const renamed = await server.request("textDocument/rename", { textDocument: { uri }, position: { line: 4, character: 1 }, newName: "Z" });
    assert.equal(renamed.changes[uri].length, 2);

    // Break the program: the bracket is never closed.
    server.send({ method: "textDocument/didChange", params: { textDocument: { uri, version: 2 }, contentChanges: [{ text: "1 [ 2 !\n$$\n" }] } });
    const broken = await server.receive();
    assert.equal(broken.params.diagnostics.length, 1);
    assert.equal(broken.params.diagnostics[0].code, "unclosed-bracket");
    assert.equal(broken.params.diagnostics[0].severity, 1);

    assert.equal(await server.request("shutdown", null), null);
    server.send({ method: "exit" });
    assert.equal(await server.exited, 0);
  });
}
