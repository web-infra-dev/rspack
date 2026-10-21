import fs from "node:fs";
import path from "node:path";
import { promisify } from "node:util";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
const cases = [false, true].map(newCache => {
  let rebuildStage;
  let earlySession;
  let dependencyBuilds = 0;
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
      const dependency = path.join(root, "dep.js");
      fs.writeFileSync(dependency, 'export const value = { label: "dep-v1" };');
      // Make the first build's snapshot eligible for reuse during finishModules.
      fs.utimesSync(dependency, 1, 1);
      fs.writeFileSync(path.join(root, "build-dependency.js"), 'module.exports = "first";');
      return {
        context: root,
        entry: "./entry.js",
        devtool: "source-map",
        incremental: false,
        stats: { cacheInfo: true },
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
                if (module.resource === dependency) dependencyBuilds++;
              });
              const rebuild = async stage => {
                if (stage !== rebuildStage) return;
                rebuildStage = undefined;
                const module = [...compilation.modules].find(item => item.resource === dependency);
                expect(module).toBeDefined();
                await promisify(compilation.rebuildModule.bind(compilation))(module);
              };
              compilation.hooks.finishModules.tapPromise("CacheModuleBuildsTest", () => {
                const { cacheInfo } = compilation.getStats().toJson({ all: false, cacheInfo: true });
                earlySession = cacheInfo.persistentSessionInitialization;
                return rebuild("finishModules");
              });
              compilation.hooks.afterSeal.tapPromise("CacheModuleBuildsTest", () => rebuild("afterSeal"));
            });
          }
        }]
      };
    },
    compiler(_context, compiler) {
      compiler.outputFileSystem = fs;
    },
    async build(context, compiler) {
      const manager = context.getCompiler();
      const root = compiler.context;
      const build = async (expected, status, reason = null) => {
        dependencyBuilds = 0;
        const stats = await manager.build();
        expect(stats.hasErrors()).toBe(false);
        const cacheInfo = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
        expect(cacheInfo.moduleBuilds).toEqual(expected);
        expect(cacheInfo.persistent).toEqual(newCache ? { status: "unknown", reason: null } : { status, reason });
        expect(cacheInfo.persistentSessionInitialization).toEqual(newCache ? { status, reason } : null);
        expect(earlySession).toEqual(cacheInfo.persistentSessionInitialization);
        expect(rebuildStage).toBeUndefined();
        return ["bundle.js", "bundle.js.map"].map(file =>
          fs.readFileSync(path.join(root, "output", file), "utf8")
        );
      };
      const restart = async () => {
        await manager.close();
        manager.createCompiler().outputFileSystem = fs;
      };

      if (newCache) rebuildStage = "finishModules";
      const cold = await build({ reused: 0, total: 2 }, "cold");
      expect(dependencyBuilds).toBe(1);
      expect(cold[0]).toContain("dep-v1");
      if (newCache) expect(await build(null, "cold")).toEqual(cold);
      await restart();
      expect(await build({ reused: 2, total: 2 }, "valid")).toEqual(cold);
      await restart();
      const dependency = path.join(root, "dep.js");
      fs.writeFileSync(dependency, 'export const value = { label: "dependency-v2" };');
      fs.utimesSync(dependency, 2, 2);
      const changed = await build({ reused: 1, total: 2 }, "valid");
      expect(changed[0]).toContain("dependency-v2");
      expect(changed[1]).not.toEqual(cold[1]);
      await restart();
      expect(await build({ reused: 2, total: 2 }, "valid")).toEqual(changed);
      await restart();
      rebuildStage = "finishModules";
      expect(await build({ reused: newCache ? 2 : 1, total: 2 }, "valid")).toEqual(changed);
      expect(dependencyBuilds).toBe(newCache ? 0 : 1);
      await restart();
      if (!newCache) {
        const cacheRoot = path.join(root, "cache");
        const cacheDirectory = fs.readdirSync(cacheRoot).find(entry =>
          fs.existsSync(path.join(cacheRoot, entry, "snapshot_file", "0.pack"))
        );
        expect(cacheDirectory).toBeDefined();
        fs.writeFileSync(path.join(cacheRoot, cacheDirectory, "snapshot_file", "0.pack"), "corrupt\n");
        expect(await build({ reused: 0, total: 2 }, "error", "recovery")).toEqual(changed);
        await restart();
      }
      fs.writeFileSync(path.join(root, "build-dependency.js"), 'module.exports = "changed dependency";');
      expect(await build({ reused: 0, total: 2 }, "invalidated", "buildDependencies")).toEqual(changed);
      await restart();
      expect(await build({ reused: 2, total: 2 }, "valid")).toEqual(changed);
      manager.mergeOptions({ cache: { type: "persistent", version: "second" } });
      await restart();
      expect(await build({ reused: 0, total: 2 }, "invalidated", "version")).toEqual(changed);
      await restart();
      rebuildStage = "afterSeal";
      expect(await build(null, "valid")).toEqual(changed);
      await build(null, newCache ? "valid" : "unknown");
      await restart();
      fs.writeFileSync(path.join(root, "entry.js"), 'console.log("no dependency");');
      const removed = await build({ reused: 0, total: 1 }, "valid");
      expect(removed[0]).not.toContain("dependency-v2");
    }
  };
});

cases.push({
  description: "counts external modules in the main module graph",
  options(context) {
    const location = context.getDist("external-cache");
    fs.rmSync(location, { recursive: true, force: true });
    return {
      entry: ["./a.js", "external"],
      stats: { cacheInfo: true },
      externals: { external: "commonjs external" },
      cache: { type: "persistent", storage: { type: "filesystem", location } },
      experiments: { newCache: false }
    };
  },
  async build(context) {
    const stats = await context.getCompiler().build();
    expect(stats.hasErrors()).toBe(false);
    expect(stats.toJson({ all: false, cacheInfo: true }).cacheInfo.moduleBuilds).toEqual({ reused: 0, total: 2 });
  }
});

export default cases;
