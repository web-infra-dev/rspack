import fs from "node:fs";
import path from "node:path";
import { CopyRspackPlugin } from "@rspack/core";
import { checkCacheCounters } from "./helpers/cache-counters.mjs";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  { name: "disabled", cache: false, newCache: false, stats: { cacheInfo: true } },
  { name: "memory", cache: { type: "memory" }, newCache: false, stats: { cacheInfo: true } },
  { name: "persistent", cache: { type: "persistent" }, newCache: false, stats: { cacheInfo: false }, suffix: "stats-disabled" },
  { name: "persistent", cache: { type: "persistent" }, newCache: true, stats: { all: true } },
  { name: "persistent", cache: { type: "persistent" }, newCache: { module: false }, stats: { cacheInfo: true }, suffix: "module-disabled" }
].map(({ name, cache, newCache, stats, suffix = "" }) => ({
  description: `reports ${name} cache mode ${suffix}`,
  options(context) {
    const root = context.getDist(`${name}-${suffix}`);
    fs.rmSync(root, { recursive: true, force: true });
    fs.mkdirSync(root, { recursive: true });
    fs.writeFileSync(path.join(root, "entry.js"), "export default 1;");
    fs.writeFileSync(path.join(root, "asset.txt"), "asset");
    return {
      context: root,
      entry: "./entry.js",
      mode: "production",
      stats,
      output: { path: path.join(root, "output") },
      cache,
      experiments: { newCache },
      plugins: name === "disabled" ? [new CopyRspackPlugin({ patterns: [{ from: "asset.txt" }] })] : []
    };
  },
  compiler(_context, compiler) {
    compiler.outputFileSystem = fs;
  },
  async build(context) {
    const stats = await context.getCompiler().build();
    expect(stats.hasErrors()).toBe(false);
    expect(stats.toJson({ all: false }).cacheInfo).toBeUndefined();
    const cacheInfo = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
    expect(stats.toJson({ all: true }).cacheInfo).toEqual(cacheInfo);
    checkCacheCounters(stats, cacheInfo.counters);
    if (name === "disabled") {
      expect(stats.toJson({ all: true }, true).cacheInfo).toBeUndefined();
      expect(cacheInfo.counters.some(counter => counter.label === "module code generation cache")).toBe(false);
      expect(cacheInfo.counters).toContainEqual({
        logger: "rspack.CopyRspackPlugin",
        label: "copy pattern cache",
        hit: 0,
        total: 1
      });
    }
    const { counters: _, persistentSessionInitialization, ...state } = cacheInfo;
    expect(state).toEqual(
      name === "persistent"
        ? { mode: name, persistent: { status: newCache ? "unknown" : "cold", reason: null }, moduleBuilds: null }
        : { mode: name, persistent: null, moduleBuilds: null }
    );
    if (name === "persistent" && newCache) {
      expect(persistentSessionInitialization).toEqual({ status: expect.any(String), reason: null });
    } else {
      expect(persistentSessionInitialization).toBeNull();
    }
    if (suffix === "stats-disabled") {
      const manager = context.getCompiler();
      await manager.close();
      manager.createCompiler().outputFileSystem = fs;
      const warm = await manager.build();
      expect(warm.hasErrors()).toBe(false);
      const info = warm.toJson({ all: false, cacheInfo: true }).cacheInfo;
      checkCacheCounters(warm, info.counters);
      const { counters: _, ...state } = info;
      expect(state).toEqual({
        mode: "persistent",
        persistent: { status: "valid", reason: null },
        persistentSessionInitialization: null,
        moduleBuilds: null
      });
    }
  }
}));
