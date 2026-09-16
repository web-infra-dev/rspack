const path = require("node:path");

let builds = 0;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description:
    "should preserve asset filenames after sharing modules with the build cache",
  options(testContext) {
    const context = path.resolve(
      __dirname,
      "../esmOutputCases/preserve-modules/asset"
    );
    return {
      context,
      entry: "./src/index.js",
      mode: "development",
      devtool: false,
      incremental: false,
      cache: {
        type: "persistent",
        storage: { directory: testContext.getDist("cache") }
      },
      experiments: {
        outputModule: true,
        newCache: { module: true, loader: false, codeGeneration: false }
      },
      module: { rules: [{ test: /\.png$/, type: "asset/resource" }] },
      output: {
        path: testContext.getDist("output"),
        module: true,
        filename: "[name].mjs",
        library: {
          type: "modern-module",
          preserveModules: path.join(context, "src")
        }
      },
      plugins: [
        {
          apply(compiler) {
            compiler.hooks.compilation.tap(
              "SharedAssetModuleCache",
              compilation => {
                compilation.hooks.buildModule.tap(
                  "SharedAssetModuleCache",
                  () => builds++
                );
              }
            );
          }
        }
      ]
    };
  },
  async build(context) {
    let firstBuilds;
    let firstAssets;
    for (let run = 0; run < 2; run++) {
      if (run > 0) {
        await context.closeCompiler();
        context.getCompiler().createCompiler();
      }
      const stats = await context.getCompiler().build();
      expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
      const assets = stats.compilation
        .getAssets()
        .map(asset => asset.name)
        .sort();
      expect(assets).toContain("index.png");
      expect(assets).toContain("components/button/index.png");
      if (run === 0) {
        firstBuilds = builds;
        firstAssets = assets;
        expect(firstBuilds).toBeGreaterThan(0);
      } else {
        expect(builds).toBe(firstBuilds);
        expect(assets).toEqual(firstAssets);
      }
    }
  }
};
