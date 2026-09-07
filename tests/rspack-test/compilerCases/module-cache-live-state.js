const fs = require("node:fs");
const path = require("node:path");
const { RawSource } = require("webpack-sources");

const PLUGIN = "ModuleCacheLiveStateTest";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
  { name: "memory", cache: true },
  { name: "persistent", cache: "persistent" },
  { name: "persistent-without-memory", cache: "persistent", maxMemoryGenerations: 0 },
  { name: "persistent-reopen", cache: "persistent", reopen: true },
  { name: "disabled", cache: false }
].map(({ name, cache, maxMemoryGenerations, reopen }) => {
  let root;
  let input;
  let iteration = 0;
  let expectedAssets = [];
  let built = 0;
  let succeeded = 0;
  let failBuild = false;
  let expectedBailouts;

  return {
    description: `should retain mutations of cached modules with ${name}`,
    options(context) {
      root = context.getDist(name);
      input = path.join(root, "input.js");
      fs.rmSync(root, { recursive: true, force: true });
      fs.mkdirSync(root, { recursive: true });
      fs.writeFileSync(input, 'module.exports = require("./data.json");');
      fs.writeFileSync(path.join(root, "data.json"), '{"value": 42}');
      // Keep timestamps outside the filesystem accuracy window for deterministic hits.
      const timestamp = new Date(Date.now() - 10000);
      fs.utimesSync(input, timestamp, timestamp);
      fs.utimesSync(path.join(root, "data.json"), timestamp, timestamp);
      return {
        context: root,
        entry: "./input.js",
        mode: "production",
        devtool: false,
        incremental: false,
        output: { path: path.join(root, "dist") },
        cache: cache === "persistent" ? {
          type: "persistent",
          ...(maxMemoryGenerations === undefined ? {} : { maxMemoryGenerations }),
          storage: { type: "filesystem", location: path.join(root, "cache") }
        } : cache,
        experiments: {
          newCache: {
            module: true,
            loader: false,
            codeGeneration: false,
            devtool: false,
            minimize: false
          }
        },
        module: {
          // Factory-owned functions must come from the fresh module after restoration.
          rules: [{ test: /\.json$/, type: "json", parser: { parse: JSON.parse } }]
        },
        plugins: [{
          apply(compiler) {
            compiler.hooks.compilation.tap(PLUGIN, compilation => {
              compilation.hooks.buildModule.tap(PLUGIN, module => {
                if (module.resource === input) built++;
              });
              compilation.hooks.succeedModule.tap(PLUGIN, module => {
                if (module.resource !== input) return;
                if (failBuild) throw new Error("module cache build failure");
                succeeded++;
                expect(Object.keys(module.buildInfo.assets).sort()).toEqual(expectedAssets);
                // This also runs on cache hits. The next build must observe this mutation.
                module.emitFile(`module-state-${iteration}.txt`, new RawSource(`${iteration}`));
              });
            });
          }
        }]
      };
    },
    compiler(_, compiler) {
      compiler.outputFileSystem = fs;
    },
    async build(context) {
      const manager = context.getCompiler();
      for (iteration = 0; iteration < 5; iteration++) {
        const rebuild = cache === false || iteration === 0 || iteration === 3;
        if (iteration === 3) {
          const timestamp = new Date(fs.statSync(input).mtimeMs + 1000);
          fs.writeFileSync(input, 'module.exports = require("./data.json").value + 1;');
          fs.utimesSync(input, timestamp, timestamp);
        }
        if (rebuild) expectedAssets = [];
        const before = built;
        const stats = await manager.build();
        expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
        expect(built - before).toBe(rebuild ? 1 : 0);
        expect(succeeded).toBe(iteration + 1);
        const bailouts = stats.toJson({
          all: false,
          modules: true,
          cachedModules: true,
          optimizationBailout: true
        }).modules.find(module => module.identifier === input).optimizationBailout;
        if (rebuild) expectedBailouts = bailouts;
        else expect(bailouts).toEqual(expectedBailouts);
        expectedAssets.push(`module-state-${iteration}.txt`);
        for (const asset of expectedAssets) {
          expect(stats.compilation.getAsset(asset)).toBeDefined();
        }
        if (reopen && iteration < 4) {
          await manager.close();
          manager.createCompiler().outputFileSystem = fs;
        }
      }
      // A failed make may have moved the graph out of the compilation. Cache
      // cleanup and compiler.close must preserve the original error without panicking.
      failBuild = true;
      await expect(manager.build()).rejects.toThrow("module cache build failure");
    }
  };
});
