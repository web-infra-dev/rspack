import fs from "node:fs";
import path from "node:path";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: "shares cache validation and attributes read errors to the compilation that encountered them",
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
    const unknown = { status: "unknown", reason: null };
    const build = async (status, parentState = unknown) => {
      const stats = await manager.build();
      expect(stats.hasErrors()).toBe(false);
      const parent = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
      const child = context.getValue("childCacheInfo");
      expect(parent.persistent).toEqual(parentState);
      expect(child.persistent).toEqual(unknown);
      for (const info of [parent, child]) {
        expect(info.persistentSessionInitialization).toEqual({ status, reason: null });
      }
    };
    await build("cold");
    await manager.close();
    // Damage the data table (family 0) while leaving the validator table intact.
    const cache = context.getDist("cache-session-child/cache");
    const tables = fs.readFileSync(path.join(cache, "LOG"), "utf8");
    const [, dataTable] = /^\s*0\s*\|\s*\d+\s*\|\s*(\d+) SST/m.exec(tables);
    const file = path.join(cache, `${dataTable}.sst`);
    fs.writeFileSync(file, Buffer.alloc(fs.statSync(file).size, 0xa5));
    manager.createCompiler().outputFileSystem = fs;
    await build("valid", { status: "error", reason: "recovery" });
    await build("valid");
  }
};
