import assert from "node:assert/strict";
import {
  closeCompiler,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  name: "close-error-releases-compilation",
  description: "should release the native compilation when shutdown fails",
  options: () => ({
    mode: "development",
    entry: "./entry.js",
  }),
  async compiler(_context, compiler) {
    compiler.hooks.shutdown.tapAsync("FailShutdown", (callback) => {
      callback(new Error("expected shutdown failure"));
    });
  },
  async build(_context, compiler) {
    const stats = await runCompiler(compiler);
    const innerCompilation = stats.compilation.__internal_getInner();
    await assert.rejects(closeCompiler(compiler), /expected shutdown failure/);
    assert.equal(compiler._lastCompilation, undefined);
    assert.deepEqual(stats.toJson(), {});
    assert.throws(() => innerCompilation.hash, /compiler was closed/);
  },
};
