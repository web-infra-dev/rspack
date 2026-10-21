import fs from "node:fs";
import { CopyRspackPlugin } from "@rspack/core";

function checkCacheCounters(stats, counters) {
  const logging = stats.toJson({ all: false, logging: "verbose" }).logging;
  const events = (logger, entries) => entries.flatMap(entry => {
    const nested = events(logger, entry.children ?? []);
    if (entry.type !== "cache") return nested;
    const match = /^(.*): [\d.]+% \((\d+)\/(\d+)\)$/.exec(entry.message);
    expect(match).not.toBeNull();
    return [{ logger, label: match[1], hit: Number(match[2]), total: Number(match[3]) }, ...nested];
  });
  const loggedCounters = Object.keys(logging).sort().flatMap(logger => events(logger, logging[logger].entries));
  expect(counters).toEqual(loggedCounters);
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  { mode: "disabled" },
  { mode: "memory" },
  { mode: "persistent", name: "stats-disabled", stats: { cacheInfo: false } },
  { mode: "persistent", name: "stats-all", newCache: true, stats: { all: true } },
  { mode: "persistent", name: "module-disabled", newCache: { module: false } }
].map(({ mode, name = mode, newCache = false, stats = { cacheInfo: true } }) => ({
  name,
  description: `reports ${mode} cache mode`,
  options(context) {
    let cache = mode === "disabled" ? false : { type: mode };
    if (mode === "persistent") {
      const location = context.getDist("cache");
      fs.rmSync(location, { recursive: true, force: true });
      cache = { type: "persistent", storage: { type: "filesystem", location } };
    }
    return {
      cache,
      stats,
      experiments: { newCache },
      plugins: mode === "disabled" ? [new CopyRspackPlugin({ patterns: [{ from: "abc.txt" }] })] : []
    };
  },
  async build(context, compiler) {
    const manager = context.getCompiler();
    const check = async status => {
      const stats = await manager.build();
      expect(stats.hasErrors()).toBe(false);
      const info = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
      expect(stats.toJson({ all: false }).cacheInfo).toBeUndefined();
      expect(stats.toJson({ all: true }).cacheInfo).toEqual(info);
      expect(info.mode).toBe(mode);
      expect(info.moduleBuilds).toBeNull();
      if (mode === "persistent") {
        expect(info.persistent).toEqual({ status: newCache ? "unknown" : status, reason: null });
      } else {
        expect(info.persistent).toBeNull();
      }
      expect(info.persistentSessionInitialization).toEqual(newCache ? { status, reason: null } : null);
      checkCacheCounters(stats, info.counters);

      const codegen = expect.objectContaining({ logger: "rspack.Compilation", label: "module code generation cache" });
      if (mode === "disabled") {
        expect(stats.toJson({ all: true }, true).cacheInfo).toBeUndefined();
        expect(info.counters).not.toContainEqual(codegen);
        expect(info.counters).toContainEqual({ logger: "rspack.CopyRspackPlugin", label: "copy pattern cache", hit: 0, total: 1 });
      } else if (mode === "persistent" && !newCache) {
        expect(info.counters).toContainEqual(codegen);
      }
    };
    await check("cold");
    if (stats.cacheInfo === false) {
      await manager.close();
      manager.createCompiler().outputFileSystem = compiler.outputFileSystem;
      await check("valid");
    }
  }
}));
