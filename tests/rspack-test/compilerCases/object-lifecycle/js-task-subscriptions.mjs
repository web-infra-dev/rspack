import assert from "node:assert/strict";
import path from "node:path";
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import { closeCompiler, createGCTracker, runCompiler } from "./helpers.mjs";

const tracker = createGCTracker();
const parallel = process.argv.includes("--parallel");

function createCompiler(name) {
  const compiler = rspack({
    name,
    context: import.meta.dirname,
    mode: "development",
    entry: "./entry.js",
    cache: false,
    output: { path: "/", filename: `${name}.js` },
    module: {
      rules: [{ test: /entry\.js$/, use: [{ loader: path.join(import.meta.dirname, "subscription-loader.cjs"), parallel: parallel ? { maxWorkers: 1 } : false, options: { name, parallel } }] }],
    },
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());
  compiler.unsafeFastDrop = process.argv.includes("--unsafe-fast-drop");
  tracker.track(compiler, name);
  return compiler;
}

async function build(compiler) {
  const stats = await runCompiler(compiler);
  assert.equal(stats.hasErrors(), false, stats.toString({ all: false, errors: true }));
  const output = compiler.outputFileSystem.readFileSync(`/${compiler.options.name}.js`, "utf8");
  assert.ok(output.includes(`subscription:${compiler.options.name}`));
}

async function main() {
  let first = createCompiler("first");
  let second = createCompiler("second");
  await Promise.all([build(first), build(second)]);
  await closeCompiler(first);
  first = undefined;
  await tracker.waitForCollection("first");
  // Releasing one compiler must not cancel the shared main subscription.
  await build(second);
  await closeCompiler(second);
  second = undefined;
  await tracker.waitForCollection("second");

  // The dispatcher survives, but the old subscription and its TSFN may retire.
  let replacement = createCompiler("replacement");
  await build(replacement);
  await closeCompiler(replacement);
  replacement = undefined;
  await tracker.waitForCollection("replacement");
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
