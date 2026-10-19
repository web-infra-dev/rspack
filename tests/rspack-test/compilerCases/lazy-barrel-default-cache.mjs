import fs from "node:fs";
import path from "node:path";
import { rspack } from "@rspack/core";

let restoredBarrels = 0;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description:
    "should keep unused reexports lazy when restoring a barrel with a forwarded default",
  options(context) {
    const root = context.getDist("src");
    fs.mkdirSync(root, { recursive: true });
    const files = {
      "index.js": `import value from "./barrel.js"; export default value();`,
      "barrel.js": `export { default } from "./value.js"; export { extra } from "./extra.js";`,
      "value.js": `export default () => "value";`,
      // An unrelated lazy reexport must not start failing after a cache hit.
      "extra.js": `export { extra } from "./not-installed.js";`,
    };
    for (const [name, source] of Object.entries(files)) {
      fs.writeFileSync(path.join(root, name), source);
    }
    return {
      context: root,
      entry: "./index.js",
      mode: "development",
      devtool: false,
      incremental: false,
      experiments: { newCache: true },
      cache: {
        type: "persistent",
        storage: { directory: context.getDist("cache") },
      },
      lazyCompilation: { entries: false, imports: true },
      module: { rules: [{ test: /barrel\.js$/, sideEffects: false }] },
      optimization: { usedExports: false, sideEffects: "flag" },
      output: {
        path: context.getDist("output"),
        filename: "main.js",
        library: { type: "commonjs2" },
      },
      plugins: [
        {
          apply(compiler) {
            rspack.lazyCompilationMiddleware(compiler);
            compiler.hooks.compilation.tap(
              "LazyBarrelDefaultCache",
              compilation => {
                compilation.hooks.stillValidModule.tap(
                  "LazyBarrelDefaultCache",
                  module => {
                    if (path.basename(module.identifier()) === "barrel.js") {
                      restoredBarrels++;
                    }
                  },
                );
              },
            );
          },
        },
      ],
    };
  },
  async build(context) {
    const manager = context.getCompiler();
    for (let run = 0; run < 2; run++) {
      if (run > 0) {
        // A new compiler restores the module cache instead of the incremental graph.
        await manager.close();
        manager.createCompiler();
      }
      const stats = await manager.build();
      expect(restoredBarrels).toBe(run);
      expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
      expect(
        [...stats.compilation.modules].map(module =>
          path.basename(module.identifier()),
        ),
      ).not.toContain("extra.js");
      const output = { exports: {} };
      new Function(
        "module",
        stats.compilation.getAsset("main.js").source.source(),
      )(output);
      expect(output.exports.default).toBe("value");
    }
  },
};
