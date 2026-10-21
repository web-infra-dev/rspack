import fs from "node:fs";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: "reports an unavailable cache database without failing the build",
  options(context) {
    fs.writeFileSync(context.getDist("blocked"), "not a directory");
    return {
      experiments: { newCache: { module: true } },
      cache: { type: "persistent", storage: { type: "filesystem", location: context.getDist("blocked/cache") } }
    };
  },
  async build(context) {
    const stats = await context.getCompiler().build();
    expect(stats.hasErrors()).toBe(false);
    const info = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
    expect(info.persistent).toEqual({ status: "unknown", reason: null });
    expect(info.persistentSessionInitialization).toEqual({ status: "error", reason: null });
  }
};
