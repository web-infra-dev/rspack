import fs from "node:fs";
import path from "node:path";
import { CopyRspackPlugin } from "@rspack/core";
import { checkCacheCounters } from "./helpers/cache-counters.mjs";

const persistent = (status, reason = null) => ({
  mode: "persistent",
  persistent: { status, reason },
  moduleBuilds: null,
  persistentSessionInitialization: null
});

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  {
    description: "reports persistent cache validation in JSON stats",
    options(context) {
      const root = context.getDist("cache-info");
      fs.rmSync(root, { recursive: true, force: true });
      fs.mkdirSync(root, { recursive: true });
      fs.writeFileSync(path.join(root, "entry.js"), "export default 1;");
      fs.writeFileSync(path.join(root, "build-dependency"), "first");
      context.setValue("root", root);
      return {
        context: root,
        entry: "./entry.js",
        mode: "production",
        devtool: "source-map",
        incremental: false,
        output: { path: path.join(root, "output"), filename: "bundle.js" },
        experiments: { newCache: false },
        cache: {
          type: "persistent",
          version: "first",
          buildDependencies: [path.join(root, "build-dependency")],
          storage: { type: "filesystem", location: path.join(root, "cache") }
        }
      };
    },
    compiler(_context, compiler) {
      compiler.outputFileSystem = fs;
    },
    async build(context) {
      const manager = context.getCompiler();
      const output = path.join(context.getValue("root"), "output");
      let baseline;
      const info = async () => {
        const stats = await manager.build();
        expect(stats.hasErrors()).toBe(false);
        const artifacts = () => ["bundle.js", "bundle.js.map"].map(file =>
          fs.readFileSync(path.join(output, file), "utf8")
        );
        const before = artifacts();
        expect(stats.toJson({ all: false }).cacheInfo).toBeUndefined();
        const cacheInfo = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
        expect(stats.toJson({ all: true }).cacheInfo).toEqual(cacheInfo);
        checkCacheCounters(stats, cacheInfo.counters);
        expect(cacheInfo.counters).toContainEqual(expect.objectContaining({
          logger: "rspack.Compilation",
          label: "module code generation cache"
        }));
        expect(artifacts()).toEqual(before);
        if (baseline) expect(before).toEqual(baseline);
        else baseline = before;
        const { counters: _, ...state } = cacheInfo;
        return state;
      };
      const restart = async version => {
        await manager.close();
        const options = manager.getOptions();
        manager.setOptions({ ...options, cache: { ...options.cache, version } });
        manager.createCompiler().outputFileSystem = fs;
      };

      expect(await info()).toEqual(persistent("cold"));
      await restart("first");
      expect(await info()).toEqual(persistent("valid"));
      await restart("first");
      const cacheRoot = path.join(context.getValue("root"), "cache");
      const cacheDirectory = fs.readdirSync(cacheRoot).find(entry =>
        fs.existsSync(path.join(cacheRoot, entry, "snapshot_file", "0.pack"))
      );
      expect(cacheDirectory).toBeDefined();
      fs.writeFileSync(path.join(cacheRoot, cacheDirectory, "snapshot_file", "0.pack"), "corrupt\n");
      expect(await info()).toEqual(persistent("error", "recovery"));
      await restart("first");
      fs.writeFileSync(path.join(context.getValue("root"), "build-dependency"), "changed");
      expect(await info()).toEqual(persistent("invalidated", "buildDependencies"));
      await restart("second");
      expect(await info()).toEqual(persistent("invalidated", "version"));
      expect(await info()).toEqual(persistent("unknown"));
    }
  },
  ...[
    { name: "disabled", cache: false, newCache: false, stats: { cacheInfo: true } },
    { name: "memory", cache: { type: "memory" }, newCache: false, stats: { cacheInfo: true } },
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
      const cacheInfo = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
      checkCacheCounters(stats, cacheInfo.counters);
      if (name === "disabled") {
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
          ? { mode: name, persistent: { status: "unknown", reason: null }, moduleBuilds: null }
          : { mode: name, persistent: null, moduleBuilds: null }
      );
      if (name === "persistent") {
        expect(persistentSessionInitialization).toEqual({ status: expect.any(String), reason: null });
      } else {
        expect(persistentSessionInitialization).toBeNull();
      }
    }
  }))
];
