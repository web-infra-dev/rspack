import assert from "node:assert/strict";
import rspack from "@rspack/core";
import {
  closeCompiler,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

function releaseCompilation(compiler) {
  return new Promise((resolve, reject) => {
    compiler.releaseCompilation((err) => {
      if (err) return reject(err);
      resolve();
    });
  });
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  name: "release-compilation-after-close",
  description:
    "should keep Stats readable after close until releaseCompilation",
  options: () => ({
    mode: "development",
    entry: "./entry.js",
    output: { filename: "bundle.js" },
  }),
  async build(_context, compiler) {
    const stats = await runCompiler(compiler);
    assert.equal(stats.hasErrors(), false);

    await closeCompiler(compiler);

    assert.doesNotThrow(() => {
      stats.toJson({ all: false, assets: true });
    });

    await releaseCompilation(compiler);
    await releaseCompilation(compiler);

    assert.throws(
      () => {
        stats.toJson({ all: false, assets: true });
      },
      /Unable to access `Stats`/,
    );

    const second = rspack({
      mode: "development",
      context: compiler.context,
      entry: "./entry.js",
      output: { filename: "bundle-2.js" },
    });

    await new Promise((resolve, reject) => {
      second.run((err, secondStats) => {
        if (err) return reject(err);
        assert.equal(secondStats?.hasErrors(), false);
        second.close((closeErr) => {
          if (closeErr) return reject(closeErr);
          resolve();
        });
      });
    });
  },
};
