const { RawSource } = require("webpack-sources");

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = ["memory", "persistent"].map(type => {
  const pluginName = `SharedModuleCache-${type}`;
  let builds = 0;
  let completed = 0;
  let builtModule;

  return {
    description: `should isolate build state from shared module mutations with ${type} cache`,
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
                  builtModule = module;
                  if (completed === 0) {
                    module.emitFile("built.txt", new RawSource("build output"));
                  }
                });
                compilation.hooks.seal.tap(pluginName, () => {
                  // A wrapper retained from a build hook must not expose a mutable
                  // pointer once the graph and cache share the published module.
                  expect(() =>
                    builtModule.emitFile("late.txt", new RawSource("late"))
                  ).toThrow(/Unable to modify/);
                  const module = [...compilation.modules].find(module =>
                    module.resource?.endsWith("/d.js")
                  );
                  // A cache hit must retain the fresh factory's metadata, even
                  // though the previous compilation changed the cached module.
                  expect(module.factoryMeta.sideEffectFree).toBe(true);
                  module.factoryMeta = { sideEffectFree: false };
                  expect(module.factoryMeta.sideEffectFree).toBe(false);
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
        expect(stats.compilation.getAsset("late.txt")).toBeUndefined();
      }
    },
    async check() {
      expect(builds).toBe(1);
      expect(completed).toBe(type === "memory" ? 1 : 2);
    }
  };
});
