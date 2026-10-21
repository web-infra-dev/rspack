import fs from "node:fs";
import path from "node:path";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: "shares cache validation and attributes read errors to the compilation that encountered them",
  options(context) {
    const location = context.getDist("cache");
    fs.rmSync(location, { recursive: true, force: true });
    return {
      incremental: false,
      experiments: { newCache: { module: true } },
      cache: { type: "persistent", storage: { type: "filesystem", location } },
      plugins: [{
        apply(compiler) {
          compiler.hooks.make.tapAsync("CacheSessionChildTest", (compilation, callback) => {
            const child = compilation.createChildCompiler("child", { filename: "child.js" }, [
              new compiler.rspack.EntryPlugin(compiler.context, "./b.js", { name: "child" })
            ]);
            child.runAsChild(callback);
          });
        }
      }]
    };
  },
  async build(context, compiler) {
    const manager = context.getCompiler();
    const unknown = { status: "unknown", reason: null };
    const build = async (status, parentState = unknown) => {
      const stats = await manager.build();
      expect(stats.hasErrors()).toBe(false);
      const report = stats.toJson({ all: false, children: true, cacheInfo: true });
      const parent = report.cacheInfo;
      const child = report.children[0].cacheInfo;
      expect(parent.persistent).toEqual(parentState);
      expect(child.persistent).toEqual(unknown);
      for (const info of [parent, child]) {
        expect(info.persistentSessionInitialization).toEqual({ status, reason: null });
      }
    };
    await build("cold");
    await manager.close();
    // Damage the data table (family 0) while leaving the validator table intact.
    const cache = context.getDist("cache");
    const tables = fs.readFileSync(path.join(cache, "LOG"), "utf8");
    const [, dataTable] = /^\s*0\s*\|\s*\d+\s*\|\s*(\d+) SST/m.exec(tables);
    const file = path.join(cache, `${dataTable}.sst`);
    fs.writeFileSync(file, Buffer.alloc(fs.statSync(file).size, 0xa5));
    manager.createCompiler().outputFileSystem = compiler.outputFileSystem;
    await build("valid", { status: "error", reason: "recovery" });
    await build("valid");
  }
};
