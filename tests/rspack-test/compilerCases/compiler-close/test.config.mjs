import assert from "node:assert/strict";
import {
  runCompiler,
  closeCompiler,
} from "@rspack/test-tools/helper/lifecycle";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  name: "closed-api",
  description: "should report a clear error when APIs are called after close",
  options: () => ({
    mode: "development",
    entry: "./entry.js",
    output: { filename: () => "bundle.js" },
  }),
  async build(_context, compiler) {
    let stats;
    let innerCompilation;
    let nativeStats;
    let entries;
    let entryData;
    let retainedModule;
    try {
      stats = await runCompiler(compiler);
      innerCompilation = stats.compilation.__internal_getInner();
      nativeStats = innerCompilation.getStats();
      entries = innerCompilation.entries;
      entryData = entries.get("main");
      assert(entryData);
      retainedModule = stats.compilation.modules.values().next().value;
      assert(retainedModule);
      assert.notDeepEqual(stats.toJson({ all: false, hash: true }), {});
      const shutdown = compiler.cache.shutdown.bind(compiler.cache);
      compiler.cache.shutdown = (callback) => {
        assert.equal(compiler._lastCompilation, undefined);
        assert.deepEqual(stats.toJson(), {});
        shutdown(callback);
      };
    } finally {
      await Promise.all([closeCompiler(compiler), closeCompiler(compiler)]);
    }
    assert.equal(compiler._lastCompilation, undefined);
    assert.deepEqual(stats.toJson(), {});
    assert.equal(stats.toString(), "");
    assert.equal(stats.hash, null);
    assert.equal(stats.hasErrors(), false);
    assert.equal(stats.hasWarnings(), false);
    assert.throws(() => innerCompilation.hash, /compiler was closed/);
    assert.throws(() => nativeStats.getLogging(0), /compiler was closed/);
    assert.throws(() => entries.size, /compiler was closed/);
    assert.throws(() => entryData.dependencies, /compiler was closed/);
    assert.throws(() => retainedModule.readableIdentifier(), /compiler was closed/);
    await closeCompiler(compiler);
    await assert.rejects(runCompiler(compiler), (error) => {
      assert(
        error.message.includes(
          "Rspack compiler has already been closed by `compiler.close()`. Do not call Rspack compiler APIs after close; create a new compiler instead.",
        ),
      );
      return true;
    });
  },
};
