import fs from "node:fs";
import path from "node:path";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: "reports shared cache session initialization for parent and child compilers",
  options(context) {
    const root = context.getDist("cache-session-child");
    fs.rmSync(root, { recursive: true, force: true });
    fs.mkdirSync(root, { recursive: true });
    fs.writeFileSync(path.join(root, "entry.js"), 'console.log("parent");');
    fs.writeFileSync(path.join(root, "child.js"), 'console.log("child");');
    return {
      context: root,
      entry: "./entry.js",
      mode: "production",
      incremental: false,
      output: { path: path.join(root, "output"), filename: "parent.js" },
      experiments: { newCache: { module: true } },
      cache: { type: "persistent", storage: { type: "filesystem", location: path.join(root, "cache") } },
      plugins: [{
        apply(compiler) {
          compiler.hooks.make.tapAsync("CacheSessionChildTest", (compilation, callback) => {
            const child = compilation.createChildCompiler("child", { filename: "child.js" }, [
              new compiler.rspack.EntryPlugin(root, "./child.js", { name: "child" })
            ]);
            child.runAsChild((error, _entries, childCompilation) => {
              if (error) return callback(error);
              const info = childCompilation.getStats().toJson({ all: false, cacheInfo: true }).cacheInfo;
              context.setValue("childCacheInfo", info);
              callback();
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
    const build = async status => {
      const stats = await manager.build();
      expect(stats.hasErrors()).toBe(false);
      const parent = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
      const child = context.getValue("childCacheInfo");
      for (const info of [parent, child]) {
        expect(info.persistent).toEqual({ status: "unknown", reason: null });
        expect(info.persistentSessionInitialization).toEqual({ status, reason: null });
      }
    };
    await build("cold");
    await manager.close();
    manager.createCompiler().outputFileSystem = fs;
    await build("valid");
  }
};
