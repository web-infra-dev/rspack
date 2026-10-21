import { rspack } from "@rspack/core";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description:
    "should keep unused reexports lazy when restoring a barrel with a forwarded default",
  options(context) {
    const barrel = context.getSource("barrel.js");
    return {
      context: context.getSource(),
      entry: "./index.js",
      mode: "development",
      devtool: false,
      incremental: false,
      experiments: { newCache: true },
      cache: {
        type: "persistent",
        storage: { directory: context.getDist("cache") },
      },
      // Cache restoration must not depend on checkout times or filesystem precision.
      snapshot: { module: { timestamp: false, hash: true } },
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
            compiler.hooks.compilation.tap("LazyBarrel", compilation => {
              const events = { built: 0, restored: 0 };
              context.setValue("barrelCacheEvents", events);
              compilation.hooks.buildModule.tap("LazyBarrel", module => {
                if (module.identifier() === barrel) {
                  events.built++;
                }
              });
              compilation.hooks.stillValidModule.tap("LazyBarrel", module => {
                if (module.identifier() === barrel) {
                  events.restored++;
                }
              });
            });
          },
        },
      ],
    };
  },
  async build(context) {
    const manager = context.getCompiler();
    for (let run = 0; run < 2; run++) {
      if (run > 0) {
        // close() flushes the persistent cache before a fresh compiler reads it.
        await manager.close();
        manager.createCompiler();
      }
      const stats = await manager.build();
      expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
      expect(context.getValue("barrelCacheEvents")).toEqual(
        run === 0 ? { built: 1, restored: 0 } : { built: 0, restored: 1 },
      );
      expect(
        [...stats.compilation.modules].some(
          module => module.identifier() === context.getSource("extra.js"),
        ),
      ).toBe(false);
      const output = { exports: {} };
      new Function(
        "module",
        stats.compilation.getAsset("main.js").source.source(),
      )(output);
      expect(output.exports.default).toBe("value");
    }
  },
};
