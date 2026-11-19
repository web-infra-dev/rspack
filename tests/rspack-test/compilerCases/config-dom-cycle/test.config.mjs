import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
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
  description:
    "checks cycles through target option types and accepts shared references",
  async run() {
    const parser = {};
    parser.worker = parser;
    const generator = {};
    generator.dataUrl = generator;
    // The source graph is cyclic, but the worker target only reads alias/url.
    for (const module of [
      { parser: { javascript: parser } },
      { rules: [{ parser }] },
      { rules: [{ type: "asset/inline", generator }] },
      // This final type does not read worker at all.
      { rules: [{ parser }, { type: "asset/source" }] },
    ]) {
      const compiler = rspack({
        mode: "none",
        entry: fileURLToPath(new URL("./index.js", import.meta.url)),
        module,
      });
      try {
        const stats = await runCompiler(compiler);
        assert.equal(stats.hasErrors(), false, stats.toString());
      } finally {
        await closeCompiler(compiler);
      }
    }

    const resolve = {};
    resolve.byDependency = { esm: { byDependency: { commonjs: resolve } } };
    for (const options of [
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
      /Invalid value for option 'restrictions'/,
    );
    assert.throws(
      () => rawResolver([{ extensions: [1] }]),
      /Configuration graph reference is out of bounds/,
    );
    // An unused cycle is harmless even when supplied directly to the binding.
    assert.doesNotThrow(() => rawResolver([{ unused: [0] }]));
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
