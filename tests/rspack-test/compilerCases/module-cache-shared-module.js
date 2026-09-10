const { RawSource } = require("webpack-sources");

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = ["memory", "persistent"].map(type => {
  const pluginName = `SharedModuleCache-${type}`;
  let builds = 0;
  let succeeded = 0;
  let reused = 0;
  let completed = 0;

  return {
    description: `should restore modules with fresh factory metadata using ${type} cache`,
    options(context) {
      return {
        context: context.getSource(),
        entry: "./d.js",
        mode: "development",
        devtool: false,
        output: { path: context.getDist("output") },
        incremental: false,
        cache:
          type === "memory"
            ? true
            : {
                type: "persistent",
                storage: { directory: context.getDist("cache") }
              },
        experiments: {
          newCache: {
            module: true,
            loader: false,
            codeGeneration: false,
            devtool: false,
            minimize: false
          }
        },
        module: { rules: [{ test: /d\.js$/, sideEffects: false }] },
        plugins: [
          {
            apply(compiler) {
              compiler.hooks.compilation.tap(pluginName, compilation => {
                compilation.hooks.buildModule.tap(pluginName, () => builds++);
                compilation.hooks.succeedModule.tap(pluginName, module => {
                  succeeded++;
                  expect(module.factoryMeta.sideEffectFree).toBe(true);
                  module.factoryMeta = { sideEffectFree: false };
                  expect(module.factoryMeta.sideEffectFree).toBe(false);
                  module.emitFile("built.txt", new RawSource("build output"));
                });
                compilation.hooks.stillValidModule.tap(pluginName, module => {
                  reused++;
                  // Cache hits receive the fresh factory's metadata, even after
                  // the previous build hook changed the cached module.
                  expect(module.factoryMeta.sideEffectFree).toBe(true);
                  expect(module.resource?.replace(/^.*[\\/]/, "")).toBe("d.js");
                });
                compilation.hooks.seal.tap(pluginName, () => {
                  const module = [...compilation.modules].find(
                    module => module.resource?.replace(/^.*[\\/]/, "") === "d.js"
                  );
                  expect(module.originalSource().source()).toContain(
                    "module.exports"
                  );
                  completed++;
                });
              });
            }
          }
        ]
      };
    },
    async build(context) {
      const runs = type === "memory" ? 1 : 2;
      for (let run = 0; run < runs; run++) {
        if (run > 0) {
          // JS run() uses the rebuild path after the initial compilation;
          // a fresh compiler exercises the persisted module build cache.
          await context.closeCompiler();
          context.getCompiler().createCompiler();
        }
        const stats = await context.getCompiler().build();
        expect(stats.hasErrors()).toBe(false);
        expect(stats.compilation.getAsset("built.txt").source.source()).toBe(
          "build output"
        );
      }
    },
    async check() {
      expect(builds).toBe(1);
      expect(succeeded).toBe(1);
      expect(reused).toBe(type === "memory" ? 0 : 1);
      expect(completed).toBe(type === "memory" ? 1 : 2);
    }
  };
});
