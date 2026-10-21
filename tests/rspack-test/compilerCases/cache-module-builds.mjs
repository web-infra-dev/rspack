import fs from "node:fs";
import path from "node:path";
import { promisify } from "node:util";

const statsOptions = { all: false, cacheInfo: true };

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [false, true].map(newCache => ({
  name: newCache ? "new" : "legacy",
  description: "counts module builds across compiler restarts",
  options(context) {
    const dependency = context.getDist("dep.js");
    fs.writeFileSync(context.getDist("entry.js"), 'import { value } from "./dep.js"; console.log(value);');
    fs.writeFileSync(dependency, 'export const value = { label: "dep-v1" };');
    // A past timestamp lets finishModules restore the module built in the same compilation.
    fs.utimesSync(dependency, 1, 1);
    fs.writeFileSync(context.getDist("config.js"), "module.exports = 1;");
    return {
      context: context.getDist(),
      entry: "./entry.js",
      externals: { external: "commonjs external" },
      devtool: "source-map",
      incremental: false,
      output: { path: context.getDist("output") },
      experiments: { newCache },
      stats: { cacheInfo: true },
      cache: {
        type: "persistent",
        buildDependencies: [context.getDist("config.js")],
        storage: { type: "filesystem", location: context.getDist("cache") }
      }
    };
  },
  async build(context) {
    const manager = context.getCompiler();
    const dependency = context.getDist("dep.js");
    const assets = () => ["main.js", "main.js.map"].map(file => fs.readFileSync(context.getDist(`output/${file}`), "utf8"));
    const build = async (reused, status, { reason = null, forceAt, twice } = {}) => {
      const compiler = manager.getStats() ? manager.createCompiler() : manager.getCompiler();
      compiler.outputFileSystem = fs;
      const session = newCache ? { status, reason } : null;
      if (forceAt) {
        compiler.hooks.compilation.tap("CacheInfoTest", compilation => {
          let dependencyBuilds = 0;
          compilation.hooks.buildModule.tap("CacheInfoTest", module => {
            if (module.resource === dependency) dependencyBuilds++;
          });
          compilation.hooks[forceAt].tapPromise("CacheInfoTest", async () => {
            const { cacheInfo } = compilation.getStats().toJson(statsOptions);
            expect(cacheInfo.persistentSessionInitialization).toEqual(session);
            const module = [...compilation.modules].find(item => item.resource === dependency);
            await promisify(compilation.rebuildModule.bind(compilation))(module);
            if (forceAt === "finishModules") expect(dependencyBuilds).toBe(status === "cold" || !newCache ? 1 : 0);
          });
        });
      }
      const stats = await manager.build();
      expect(stats.hasErrors()).toBe(false);
      const { cacheInfo } = stats.toJson(statsOptions);
      expect(cacheInfo.moduleBuilds).toEqual(reused === null ? null : { reused, total: 2 });
      expect(cacheInfo.persistent).toEqual(newCache ? { status: "unknown", reason: null } : { status, reason });
      expect(cacheInfo.persistentSessionInitialization).toEqual(session);
      if (twice) {
        const next = await manager.build();
        expect(next.toJson(statsOptions).cacheInfo).toMatchObject({ moduleBuilds: null, persistentSessionInitialization: session });
      }
      await manager.close();
    };

    await build(0, "cold", { forceAt: newCache ? "finishModules" : undefined });
    const cold = assets();
    expect(cold[0]).toContain("dep-v1");
    await build(2, "valid", { twice: true });
    expect(assets()).toEqual(cold);
    fs.writeFileSync(dependency, 'export const value = { label: "dependency-v2" };');
    fs.utimesSync(dependency, 2, 2);
    await build(1, "valid");
    const changed = assets();
    expect(changed[0]).toContain("dependency-v2");
    expect(changed[1]).not.toEqual(cold[1]);
    await build(newCache ? 2 : 1, "valid", { forceAt: "finishModules" });
    expect(assets()).toEqual(changed);

    if (!newCache) {
      const cache = context.getDist("cache");
      const files = fs.readdirSync(cache, { recursive: true });
      const snapshot = files.find(file => file.endsWith(path.join("snapshot_file", "0.pack")));
      fs.writeFileSync(path.join(cache, snapshot), "corrupt");
      await build(0, "error", { reason: "recovery" });
    }
    fs.writeFileSync(context.getDist("config.js"), "module.exports = 'changed';");
    await build(0, "invalidated", { reason: "buildDependencies" });
    manager.mergeOptions({ cache: { type: "persistent", version: "second" } });
    await build(0, "invalidated", { reason: "version" });
    expect(assets()).toEqual(changed);
    await build(null, "valid", { forceAt: "afterSeal" });

    fs.writeFileSync(context.getDist("entry.js"), 'import "external";');
    await build(0, "valid"); // The final graph contains the entry and the external.
    expect(assets()[0]).not.toContain("dependency-v2");
  }
}));
