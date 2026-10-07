import fs from "node:fs";
import path from "node:path";
import { checkCacheCounters } from "./helpers/cache-counters.mjs";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
const cases = [false, true].map(newCache => {
  let forceRebuild = false;
  let forceLateRebuild = false;
  let dependencyBuilds = 0;
  let forcedRebuilds = 0;
  return {
    description: `reports module build reuse with ${newCache ? "new" : "legacy"} persistent cache`,
    options(context) {
      const root = context.getDist(newCache ? "new-module-builds" : "legacy-module-builds");
      fs.rmSync(root, { recursive: true, force: true });
      fs.mkdirSync(root, { recursive: true });
      fs.writeFileSync(
        path.join(root, "entry.js"),
        'import { value } from "./dep.js"; import { value as again } from "./dep.js"; console.log(value, again);'
      );
      fs.writeFileSync(path.join(root, "dep.js"), 'export const value = "dep-v1";');
      fs.writeFileSync(path.join(root, "build-dependency.js"), 'module.exports = "first";');
      context.setValue("root", root);
      return {
        context: root,
        entry: "./entry.js",
        mode: "production",
        devtool: "source-map",
        incremental: false,
        stats: { cacheInfo: true },
        optimization: { minimize: false },
        output: { path: path.join(root, "output"), filename: "bundle.js" },
        experiments: {
          newCache: newCache && {
            module: true,
            loader: false,
            codeGeneration: false,
            devtool: false,
            minimize: false
          }
        },
        cache: {
          type: "persistent",
          version: "first",
          buildDependencies: [path.join(root, "build-dependency.js")],
          storage: { type: "filesystem", location: path.join(root, "cache") }
        },
        plugins: [{
          apply(compiler) {
            compiler.hooks.compilation.tap("CacheModuleBuildsTest", compilation => {
              compilation.hooks.buildModule.tap("CacheModuleBuildsTest", module => {
                if (module.resource === path.join(root, "dep.js")) dependencyBuilds++;
              });
              compilation.hooks.finishModules.tapPromise("CacheModuleBuildsTest", async modules => {
                if (!forceRebuild) return;
                forceRebuild = false;
                const module = [...modules].find(item => item.resource === path.join(root, "dep.js"));
                expect(module).toBeDefined();
                await new Promise((resolve, reject) => {
                  compilation.rebuildModule(module, (error, rebuilt) => {
                    if (error) return reject(error);
                    expect(rebuilt.identifier()).toBe(module.identifier());
                    forcedRebuilds++;
                    resolve();
                  });
                });
              });
              compilation.hooks.afterSeal.tapPromise("CacheModuleBuildsTest", async () => {
                if (!forceLateRebuild) return;
                forceLateRebuild = false;
                const module = [...compilation.modules].find(item => item.resource === path.join(root, "dep.js"));
                expect(module).toBeDefined();
                await new Promise((resolve, reject) => {
                  compilation.rebuildModule(module, error => {
                    if (error) return reject(error);
                    resolve();
                  });
                });
              });
            });
          }
        }]
      };
    },
    compiler(_context, compiler) {
      compiler.outputFileSystem = fs;
    },
    async build(context) {
      const manager = context.getCompiler();
      const root = context.getValue("root");
      const artifacts = () => ["bundle.js", "bundle.js.map"].map(file =>
        fs.readFileSync(path.join(root, "output", file), "utf8")
      );
      const build = async expected => {
        dependencyBuilds = 0;
        const stats = await manager.build();
        expect(stats.hasErrors()).toBe(false);
        const cacheInfo = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
        expect(cacheInfo.moduleBuilds).toEqual(expected);
        checkCacheCounters(stats, cacheInfo.counters);
        return artifacts();
      };
      const restart = async version => {
        await manager.close();
        const options = manager.getOptions();
        manager.setOptions({ ...options, cache: { ...options.cache, version } });
        manager.createCompiler().outputFileSystem = fs;
      };

      const cold = await build({ reused: 0, total: 2 });
      expect(cold[0]).toContain("dep-v1");
      await restart("first");
      expect(await build({ reused: 2, total: 2 })).toEqual(cold);
      await restart("first");
      fs.writeFileSync(path.join(root, "dep.js"), 'export const value = "dependency-v2";');
      const changed = await build({ reused: 1, total: 2 });
      expect(changed[0]).toContain("dependency-v2");
      expect(changed[1]).not.toEqual(cold[1]);
      await restart("first");
      expect(await build({ reused: 2, total: 2 })).toEqual(changed);
      await restart("first");
      forceRebuild = true;
      const stats = await manager.build();
      expect(stats.hasErrors()).toBe(false);
      expect(forcedRebuilds).toBe(1);
      expect(dependencyBuilds).toBeLessThanOrEqual(1);
      if (!newCache) expect(dependencyBuilds).toBe(1);
      expect(stats.toJson({ all: false, cacheInfo: true }).cacheInfo.moduleBuilds).toEqual({
        reused: dependencyBuilds ? 1 : 2,
        total: 2
      });
      expect(artifacts()).toEqual(changed);
      await restart("first");
      if (!newCache) {
        const cacheRoot = path.join(root, "cache");
        const cacheDirectory = fs.readdirSync(cacheRoot).find(entry =>
          fs.existsSync(path.join(cacheRoot, entry, "snapshot_file", "0.pack"))
        );
        expect(cacheDirectory).toBeDefined();
        fs.writeFileSync(path.join(cacheRoot, cacheDirectory, "snapshot_file", "0.pack"), "corrupt\n");
        expect(await build({ reused: 0, total: 2 })).toEqual(changed);
        await restart("first");
      }
      fs.writeFileSync(path.join(root, "build-dependency.js"), 'module.exports = "changed dependency";');
      expect(await build({ reused: 0, total: 2 })).toEqual(changed);
      await restart("first");
      expect(await build({ reused: 2, total: 2 })).toEqual(changed);
      await restart("second");
      expect(await build({ reused: 0, total: 2 })).toEqual(changed);
      await restart("second");
      forceLateRebuild = true;
      expect(await build(null)).toEqual(changed);
      await build(null);
      await restart("second");
      fs.writeFileSync(path.join(root, "entry.js"), 'console.log("no dependency");');
      const removed = await build({ reused: 0, total: 1 });
      expect(removed[0]).not.toContain("dependency-v2");
    }
  };
});

cases.push({
  description: "counts external modules in the main module graph",
  options(context) {
    const root = context.getDist("external-module-builds");
    fs.rmSync(root, { recursive: true, force: true });
    fs.mkdirSync(root, { recursive: true });
    fs.writeFileSync(path.join(root, "entry.js"), 'console.log(require("external"));');
    return {
      context: root,
      entry: "./entry.js",
      mode: "production",
      stats: { cacheInfo: true },
      externals: { external: "commonjs external" },
      output: { path: path.join(root, "output") },
      cache: { type: "persistent", storage: { type: "filesystem", location: path.join(root, "cache") } },
      experiments: { newCache: false }
    };
  },
  compiler(_context, compiler) {
    compiler.outputFileSystem = fs;
  },
  async build(context) {
    const stats = await context.getCompiler().build();
    expect(stats.hasErrors()).toBe(false);
    expect(stats.toJson({ all: false, cacheInfo: true }).cacheInfo.moduleBuilds).toEqual({ reused: 0, total: 2 });
  }
});

export default cases;
