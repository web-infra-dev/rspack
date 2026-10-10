import fs from "node:fs";
import path from "node:path";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  {
    description: "reports the new persistent cache session initialization",
    options(context) {
      const root = context.getDist("cache-session-initialization");
      fs.rmSync(root, { recursive: true, force: true });
      fs.mkdirSync(root, { recursive: true });
      fs.writeFileSync(path.join(root, "entry.js"), 'console.log("entry");');
      fs.writeFileSync(path.join(root, "build-dependency"), "first");
      context.setValue("root", root);
      return {
        context: root,
        entry: "./entry.js",
        mode: "production",
        devtool: "source-map",
        incremental: false,
        output: { path: path.join(root, "output"), filename: "bundle.js" },
        experiments: { newCache: { module: true } },
        cache: {
          type: "persistent",
          version: "first",
          buildDependencies: [path.join(root, "build-dependency")],
          storage: { type: "filesystem", location: path.join(root, "cache") }
        },
        plugins: [{
          apply(compiler) {
            compiler.hooks.compilation.tap("CacheSessionTest", compilation => {
              compilation.hooks.finishModules.tap("CacheSessionTest", () => {
                const info = compilation.getStats().toJson({ all: false, cacheInfo: true }).cacheInfo;
                context.setValue("earlySession", info.persistentSessionInitialization);
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
      let baseline;
      const build = async (status, reason = null) => {
        const stats = await manager.build();
        expect(stats.hasErrors()).toBe(false);
        const info = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
        expect(info.persistent).toEqual({ status: "unknown", reason: null });
        expect(info.persistentSessionInitialization).toEqual({ status, reason });
        expect(context.getValue("earlySession")).toEqual(info.persistentSessionInitialization);
        const artifacts = ["bundle.js", "bundle.js.map"].map(file =>
          fs.readFileSync(path.join(root, "output", file), "utf8")
        );
        if (baseline) expect(artifacts).toEqual(baseline);
        else baseline = artifacts;
      };
      const restart = async (version, mutate) => {
        await manager.close();
        mutate?.();
        const options = manager.getOptions();
        manager.setOptions({ ...options, cache: { ...options.cache, version } });
        manager.createCompiler().outputFileSystem = fs;
      };

      await build("cold");
      await build("cold");
      await restart("first");
      await build("valid");
      await restart("first", () => fs.writeFileSync(path.join(root, "build-dependency"), "changed"));
      await build("invalidated", "buildDependencies");
      await restart("second");
      await build("invalidated", "version");
    }
  },
  {
    description: "reports an unavailable cache database without failing the build",
    options(context) {
      const root = context.getDist("cache-session-error");
      fs.rmSync(root, { recursive: true, force: true });
      fs.mkdirSync(root, { recursive: true });
      fs.writeFileSync(path.join(root, "entry.js"), "export default 1;");
      fs.writeFileSync(path.join(root, "blocked"), "not a directory");
      return {
        context: root,
        entry: "./entry.js",
        mode: "production",
        output: { path: path.join(root, "output") },
        experiments: { newCache: { module: true } },
        cache: { type: "persistent", storage: { type: "filesystem", location: path.join(root, "blocked", "cache") } }
      };
    },
    compiler(_context, compiler) {
      compiler.outputFileSystem = fs;
    },
    async build(context) {
      const stats = await context.getCompiler().build();
      expect(stats.hasErrors()).toBe(false);
      const info = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
      expect(info.persistent).toEqual({ status: "unknown", reason: null });
      expect(info.persistentSessionInitialization).toEqual({ status: "error", reason: null });
    }
  }
];
