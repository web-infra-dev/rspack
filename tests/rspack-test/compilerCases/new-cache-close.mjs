import path from "node:path";
import assert from "node:assert/strict";
import {
  runCompiler,
  closeCompiler,
} from "@rspack/test-tools/helper/lifecycle";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  name: "closed-api-new-cache",
  description: "should report a clear error when APIs are called after close",
  options: () => ({
    mode: "development",
    context: path.join(import.meta.dirname, "compiler-close"),
    entry: "./entry.js",
    experiments: { newCache: true },
    output: { filename: () => "bundle.js" },
  }),
  async build(_context, compiler) {
    try {
      await runCompiler(compiler);
    } finally {
      await Promise.all([closeCompiler(compiler), closeCompiler(compiler)]);
    }
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
