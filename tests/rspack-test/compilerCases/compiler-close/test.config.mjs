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
    try {
      await runCompiler(compiler);
    } finally {
      await closeCompiler(compiler);
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
