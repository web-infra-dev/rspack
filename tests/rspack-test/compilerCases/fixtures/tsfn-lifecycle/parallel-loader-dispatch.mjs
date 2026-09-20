import assert from "node:assert/strict";
import path from "node:path";
import vm from "node:vm";
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import { closeCompiler, runCompiler } from "./helpers.mjs";

const cases = path.resolve(import.meta.dirname, "../../../configCases/loader-parallel");
const output = createFsFromVolume(new Volume());
async function build(entry, loader, parallel) {
  const compiler = rspack({
    mode: "development",
    context: import.meta.dirname,
    target: "node",
    entry,
    devtool: false,
    module: { rules: [{ test: /(?:m\d|resource)\.js$/, use: [{ loader, parallel, options: {} }] }] },
    output: { path: "/", filename: "bundle.js", library: { type: "commonjs2" } },
  });
  compiler.outputFileSystem = output;
  try { return await runCompiler(compiler); }
  finally { await closeCompiler(compiler); }
}

// This process owns its pool, so other test compilers cannot set its size or
// have their active work interrupted by the deliberate worker crash below.
const entry = path.join(import.meta.dirname, "parallel-distribution-entry.cjs");
let stats = await build(entry, path.join(cases, "worker-concurrency/loader.js"), { maxWorkers: 2 });
assert.equal(stats.hasErrors(), false, stats.toString());
const context = { module: { exports: {} } };
vm.runInNewContext(output.readFileSync("/bundle.js", "utf8"), context);
assert.equal(new Set(context.module.exports).size, 2);
assert.ok(context.module.exports.every(id => id > 0));

stats = await build(path.join(import.meta.dirname, "parallel-crash-resource.js"), path.join(import.meta.dirname, "parallel-crash-loader.cjs"), true);
assert.equal(stats.hasErrors(), true);
assert.match(stats.toString(), /Parallel loader (worker exited|task was abandoned)/);

stats = await build(entry, path.join(cases, "worker-concurrency/loader.js"), true);
assert.equal(stats.hasErrors(), false, stats.toString());
