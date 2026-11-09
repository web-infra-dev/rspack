import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import { closeCompiler, createGCTracker, runCompiler } from "./helpers.mjs";

const require = createRequire(import.meta.resolve("@rspack/core"));
const binding = require("@rspack/binding");
const OriginalCompiler = binding.JsCompiler;
let checked = false;
binding.JsCompiler = new Proxy(OriginalCompiler, {
  construct(target, args) {
    const invalid = [...args];
    invalid[1] = {};
    // Fail native option conversion after transferring the subscription Arc.
    assert.throws(() => Reflect.construct(target, invalid), /Missing field|Failed to convert/);
    assert.throws(() => Reflect.construct(target, args), /unused main JS task subscription/);
    args[args.length - 1] = binding.getJsTaskSubscription(false);
    checked = true;
    return Reflect.construct(target, args);
  },
});

const tracker = createGCTracker();
async function build() {
  const compiler = rspack({
    context: import.meta.dirname,
    mode: "development",
    entry: "./entry.js",
    output: { path: "/", filename: "bundle.js" },
  });
  tracker.track(compiler, "constructor recovery");
  compiler.outputFileSystem = createFsFromVolume(new Volume());
  try {
    const stats = await runCompiler(compiler);
    assert.equal(stats.hasErrors(), false, stats.toString());
  } finally {
    await closeCompiler(compiler);
  }
}
try {
  await build();
  assert.equal(checked, true);
} finally {
  binding.JsCompiler = OriginalCompiler;
}
await tracker.waitForCollection("constructor recovery");
