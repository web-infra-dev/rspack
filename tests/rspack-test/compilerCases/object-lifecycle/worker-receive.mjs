import assert from "node:assert/strict";
import path from "node:path";
import vm from "node:vm";
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import { closeCompiler, runCompiler } from "@rspack/test-tools/helper/lifecycle";

// Own a single-worker pool: recursive imports must make progress on that worker.
const output = createFsFromVolume(new Volume());
const compiler = rspack({
  mode: "development",
  context: import.meta.dirname,
  target: "node",
  cache: false,
  devtool: "source-map",
  entry: "./receive-root.cjs",
  module: { rules: [{ test: /receive-.*\.cjs$/, use: [{
    loader: path.join(import.meta.dirname, "receive-loader.cjs"),
    parallel: process.argv.includes("--main") ? false : { maxWorkers: 1 }, options: {},
  }] }] },
  output: { path: "/", filename: "bundle.js", library: { type: "commonjs2" } },
});
compiler.outputFileSystem = output;
try {
  for (let i = 0; i < 2; i++) {
    const stats = await runCompiler(compiler);
    // The deliberately failing imported module remains a compilation error even
    // though the loader catches its rejected import.
    const errors = stats.toJson({ all: false, errors: true }).errors;
    assert.equal(errors.length, 1, stats.toString());
    assert.match(errors[0].message, /receive failure/);
    const context = { module: { exports: {} } };
    vm.runInNewContext(output.readFileSync("/bundle.js", "utf8"), context);
    assert.equal(context.module.exports, 42);
  }
} finally {
  await closeCompiler(compiler);
}
