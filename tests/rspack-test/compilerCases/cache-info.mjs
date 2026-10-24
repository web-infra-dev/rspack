import fs from "node:fs";

const state = status => status ? { status, reason: null } : null;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  { name: "disabled", mode: "disabled" },
  { name: "memory", mode: "memory" },
  { name: "stats-all", mode: "persistent", recovery: "cold", stats: { all: true } },
  { name: "module-disabled", mode: "persistent", newCache: { module: false }, recovery: "unknown", session: "cold", stats: { cacheInfo: true } },
  { name: "unavailable", mode: "persistent", newCache: true, recovery: "unknown", session: "error" }
].map(({ name, mode, newCache = false, stats, recovery, session }) => ({
  name,
  description: "reports cache state when module build tracking is unavailable",
  options(context) {
    let cache = mode === "memory";
    if (mode === "persistent") {
      const location = context.getDist("cache");
      if (name === "unavailable") fs.writeFileSync(location, "not a directory");
      cache = { type: mode, storage: { type: "filesystem", location } };
    }
    return { stats, experiments: { newCache }, cache };
  },
  check({ compilation }) {
    const stats = compilation.getStats();
    const { cacheInfo } = stats.toJson({ all: false, cacheInfo: true });
    expect(stats.toJson({ all: false }).cacheInfo).toBeUndefined();
    expect(stats.toJson({ all: true }).cacheInfo).toEqual(cacheInfo);
    expect(stats.toJson({ all: true }, true).cacheInfo).toBeUndefined();
    expect(cacheInfo).toMatchObject({
      mode,
      moduleBuilds: null,
      persistent: state(recovery),
      persistentSessionInitialization: state(session)
    });
    if (name === "stats-all") {
      expect(cacheInfo.counters).toContainEqual({
        logger: "rspack.Compilation", label: "module code generation cache", hit: 0, total: 1
      });
    }
  }
}));
