const fs = require("node:fs");
const path = require("node:path");
const { DllPlugin } = require("@rspack/core");

let root;
let timestamp;
let built = [];
let reused = [];

function writeEntry(value) {
  const filename = path.join(root, "index.js");
  fs.writeFileSync(
    filename,
    `module.exports = require("external") + "${value}";`,
  );
  fs.utimesSync(filename, timestamp, timestamp);
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description: "should recover DllModule dependencies from persistent cache",
  options(context) {
    root = context.getDist("polymorphic-cache");
    fs.rmSync(root, { recursive: true, force: true });
    fs.mkdirSync(root, { recursive: true });
    timestamp = new Date(Date.now() - 20000);
    writeEntry("initial");
    return {
      context: root,
      entry: { main: ["./index.js"] },
      target: "node",
      mode: "development",
      devtool: false,
      incremental: false,
      cache: {
        type: "persistent",
        storage: { type: "filesystem", location: path.join(root, "cache") },
      },
      experiments: {
        newCache: {
          module: true,
          loader: false,
          codeGeneration: false,
          devtool: false,
          minimize: false,
        },
      },
      externals: { external: "commonjs external" },
      output: {
        path: path.join(root, "dist"),
        filename: "dll.js",
        library: { type: "commonjs2" },
      },
      optimization: { concatenateModules: false, minimize: false },
      plugins: [
        new DllPlugin({ path: path.join(root, "manifest.json") }),
        {
          apply(compiler) {
            compiler.hooks.compilation.tap(
              "PolymorphicModuleCache",
              (compilation) => {
                compilation.hooks.buildModule.tap(
                  "PolymorphicModuleCache",
                  (module) => {
                    built.push(module.identifier());
                  },
                );
                compilation.hooks.stillValidModule.tap(
                  "PolymorphicModuleCache",
                  (module) => {
                    reused.push(module.identifier());
                  },
                );
              },
            );
          },
        },
      ],
    };
  },
  compiler(_, compiler) {
    compiler.outputFileSystem = fs;
  },
  async build(context) {
    const manager = context.getCompiler();
    for (let run = 0; run < 3; run++) {
      built = [];
      reused = [];
      if (run > 0) {
        await manager.close();
        manager.createCompiler().outputFileSystem = fs;
      }
      if (run === 2) {
        timestamp = new Date(timestamp.getTime() + 1000);
        writeEntry("updated");
      }

      const stats = await manager.build();
      expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
      // DllModule overrides need_build to reuse its build; ExternalModule keeps
      // the conservative default and must rebuild even when an entry exists.
      expect(built.filter((id) => id === "dll main")).toHaveLength(
        run === 0 ? 1 : 0,
      );
      expect(reused.filter((id) => id === "dll main")).toHaveLength(
        run === 0 ? 0 : 1,
      );
      expect(built.filter((id) => id.startsWith("external "))).toHaveLength(1);
      expect(reused.filter((id) => id.startsWith("external "))).toHaveLength(0);
      expect(built.filter((id) => /[\\/]index\.js$/.test(id))).toHaveLength(
        run === 1 ? 0 : 1,
      );

      const dllModule = [...stats.compilation.modules].find(
        (module) => module.identifier() === "dll main",
      );
      expect(dllModule).toBeDefined();
      expect(dllModule.dependencies).toHaveLength(1);
      const entryModule = stats.compilation.moduleGraph.getModule(
        dllModule.dependencies[0],
      );
      expect(entryModule?.resource?.replace(/^.*[\\/]/, "")).toBe("index.js");

      const manifest = JSON.parse(
        fs.readFileSync(path.join(root, "manifest.json"), "utf8"),
      );
      const output = { exports: {} };
      new Function(
        "require",
        "module",
        "exports",
        stats.compilation.getAsset("dll.js").source.source(),
      )(
        (request) => {
          expect(request).toBe("external");
          return "external-";
        },
        output,
        output.exports,
      );
      expect(output.exports(manifest.content["./index.js"].id)).toBe(
        run === 2 ? "external-updated" : "external-initial",
      );
    }
  },
};
