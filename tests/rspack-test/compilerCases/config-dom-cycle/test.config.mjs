import assert from "node:assert/strict";
import binding from "@rspack/binding";
import { rspack } from "@rspack/core";
import {
  closeCompiler,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

const circularConfig = /Circular reference in configuration object/;

function isCircularConfig(error) {
  for (let current = error; current; current = current.cause) {
    if (circularConfig.test(current.message)) return true;
  }
  return false;
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: "reports circular DOM options and accepts shared options",
  async run() {
    const parser = {};
    parser.worker = parser;
    const resolve = {};
    resolve.byDependency = { esm: { byDependency: { commonjs: resolve } } };
    for (const options of [
      { module: { rules: [{ parser }] } },
      { resolve },
      { module: { rules: [{ resolve }] } },
    ]) {
      await assert.rejects(async () => {
        const compiler = rspack({ mode: "none", entry: {}, ...options });
        try {
          await runCompiler(compiler);
        } finally {
          await closeCompiler(compiler);
        }
      }, isCircularConfig);
    }

    const rawResolver = (nodes) =>
      new binding.JsResolverFactory(
        false,
        { byDependency: { esm: { root: 0, nodes } } },
        {},
      );
    assert.throws(
      () => rawResolver([{ byDependency: [1] }, { commonjs: [0] }]),
      circularConfig,
    );
    assert.throws(
      () => rawResolver([{ restrictions: [1] }, [[1]]]),
      circularConfig,
    );
    assert.throws(
      () => rawResolver([{ extensions: [1] }]),
      /Configuration graph reference is out of bounds/,
    );
    assert.doesNotThrow(
      () =>
        rawResolver([
          { byDependency: [1] },
          { esm: [2], commonjs: [2] },
          { extensions: [3] },
          [".js"],
        ]),
    );

    const shared = { extensions: [".js"] };
    const byDependency = { esm: shared, commonjs: shared };
    const compiler = rspack({ mode: "none", entry: {}, resolve: { byDependency } });
    try {
      await runCompiler(compiler);
    } finally {
      await closeCompiler(compiler);
    }
  },
};
