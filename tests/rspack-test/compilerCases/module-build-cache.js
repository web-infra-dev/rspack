const fs = require("node:fs");
const path = require("node:path");
const { RawSource } = require("webpack-sources");

const newCache = {
  codeGeneration: false,
  devtool: false,
  loader: false,
  minimize: false,
  module: true,
};

let mtime = Date.now() - 3_600_000;

function write(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
  // Keep fixtures outside the snapshot's filesystem timestamp accuracy window.
  const timestamp = new Date((mtime += 2_000));
  fs.utimesSync(file, timestamp, timestamp);
}

function run(compiler, modifiedFiles = []) {
  return new Promise((resolve, reject) => {
    compiler.run(
      (error, stats) => {
        if (error) return reject(error);
        if (stats.hasErrors()) {
          return reject(
            new Error(stats.toString({ all: false, errors: true })),
          );
        }
        resolve(stats);
      },
      { modifiedFiles: new Set(modifiedFiles) },
    );
  });
}

function readOutput(root) {
  const output = path.join(root, "dist");
  for (const filename of Object.keys(require.cache)) {
    if (filename.startsWith(`${output}${path.sep}`))
      delete require.cache[filename];
  }
  return require(path.join(output, "main.js"));
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [false].flatMap((cache) =>
  [false, true].map((incremental) => {
    let root;
    let built;
    const entry =
      'export { value } from "./stable.js"; export const load = () => import("./async.js").then(m => m.default);';

    return {
      description: `should preserve module build records with cache=${cache}, incremental=${incremental}`,
      options(context) {
        root = context.getDist(`cache-${cache}-incremental-${incremental}`);
        fs.rmSync(root, { recursive: true, force: true });
        write(path.join(root, "src/index.js"), entry);
        write(
          path.join(root, "src/stable.js"),
          'export const value = "stable";',
        );
        write(path.join(root, "src/async.js"), 'export default "async";');
        return {
          context: path.join(root, "src"),
          mode: "development",
          devtool: false,
          target: "async-node",
          entry: "./index.js",
          cache,
          incremental,
          experiments: { newCache },
          optimization: { concatenateModules: true },
          output: {
            path: path.join(root, "dist"),
            filename: "main.js",
            library: { type: "commonjs2" },
          },
          plugins: [
            {
              apply(compiler) {
                compiler.hooks.compilation.tap(
                  "ModuleBuildCache",
                  (compilation) => {
                    built = [];
                    compilation.hooks.buildModule.tap(
                      "ModuleBuildCache",
                      (module) => {
                        if (module.resource)
                          built.push(path.basename(module.resource));
                      },
                    );
                    compilation.hooks.succeedModule.tap(
                      "ModuleBuildCache",
                      (module) => {
                        if (
                          module.resource === path.join(root, "src/stable.js")
                        ) {
                          module.emitFile(
                            "from-hook.txt",
                            new RawSource("hook asset"),
                          );
                        }
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
      async build(_, compiler) {
        const index = path.join(root, "src/index.js");
        const stable = path.join(root, "src/stable.js");
        await run(compiler);
        expect(built.sort()).toEqual(["async.js", "index.js", "stable.js"]);
        expect(readOutput(root).value).toBe("stable");
        expect(await readOutput(root).load()).toBe("async");

        const warm = await run(compiler);
        expect(built.sort()).toEqual(
          cache || incremental ? [] : ["async.js", "index.js", "stable.js"],
        );
        expect(warm.compilation.getAsset("from-hook.txt").source.source()).toBe(
          "hook asset",
        );
        expect(await readOutput(root).load()).toBe("async");

        write(index, 'export const value = "detached";');
        await run(compiler, [index]);
        expect(built).toEqual(["index.js"]);
        expect(readOutput(root).value).toBe("detached");

        // These dependencies were removed from the previous incremental artifact.
        // A warm cache must still supply their build output when they reappear.
        write(index, entry);
        const restored = await run(compiler, [index]);
        expect(built.sort()).toEqual(
          cache ? ["index.js"] : ["async.js", "index.js", "stable.js"],
        );
        expect(
          restored.compilation.getAsset("from-hook.txt").source.source(),
        ).toBe("hook asset");
        expect(readOutput(root).value).toBe("stable");
        expect(await readOutput(root).load()).toBe("async");

        write(stable, 'export const value = "changed";');
        await run(compiler, [stable]);
        expect(built.sort()).toEqual(
          cache || incremental
            ? ["stable.js"]
            : ["async.js", "index.js", "stable.js"],
        );
        expect(readOutput(root).value).toBe("changed");
        expect(await readOutput(root).load()).toBe("async");
      },
    };
  }),
);


const mainEntry = 'export { default } from "./shared.js";';
const executorEntry = 'export { default } from "./macro.txt";';

module.exports.push(
  ...["persistent"].flatMap((cacheType) =>
    [false, true].map((executionFirst) => ({
      description: `should share ${cacheType} module builds from ${executionFirst ? "importModule" : "the main graph"} to ${executionFirst ? "the main graph" : "importModule"}`,
      options(context) {
        const root = context.getDist(
          `executor-cache-${cacheType}-${executionFirst}`,
        );
        context.setValue("root", root);
        fs.rmSync(root, { recursive: true, force: true });
        write(
          path.join(root, "src/index.js"),
          executionFirst ? executorEntry : mainEntry,
        );
        write(path.join(root, "src/macro.txt"), "");
        write(
          path.join(root, "src/shared.js"),
          'export { default } from "./leaf.js";',
        );
        write(path.join(root, "src/leaf.js"), 'export default "initial";');
        write(
          path.join(root, "macro-loader.js"),
          'module.exports = async function() { this.cacheable(false); const value = await this.importModule("./shared.js", {}); return `export default ${JSON.stringify(value.default)};`; };',
        );
        write(
          path.join(root, "count-loader.js"),
          'const fs = require("node:fs"); module.exports = function(source) { fs.appendFileSync(this.getOptions().counter, "x"); return source; };',
        );
        return {
          context: path.join(root, "src"),
          mode: "development",
          devtool: false,
          target: "node",
          entry: "./index.js",
          incremental: false,
          cache:
            cacheType === "memory"
              ? true
              : {
                  type: "persistent",
                  version: "module-executor-build-records",
                  storage: {
                    type: "filesystem",
                    directory: path.join(root, "cache"),
                  },
                },
          experiments: { newCache },
          module: {
            rules: [
              { test: /\.txt$/, loader: path.join(root, "macro-loader.js") },
              {
                test: /(?:shared|leaf)\.js$/,
                loader: path.join(root, "count-loader.js"),
                options: { counter: path.join(root, "builds.txt") },
              },
            ],
          },
          output: {
            path: path.join(root, "dist"),
            filename: "main.js",
            library: { type: "commonjs2" },
          },
        };
      },
      compiler(_, compiler) {
        compiler.outputFileSystem = fs;
      },
      async build(context, compiler) {
        const root = context.getValue("root");
        const manager = context.getCompiler();
        const builds = () =>
          fs.readFileSync(path.join(root, "builds.txt"), "utf8").length;
        await run(compiler);
        expect(readOutput(root).default).toBe("initial");
        expect(builds()).toBe(2);

        let activeCompiler = compiler;
        if (cacheType === "persistent") {
          await manager.close();
          activeCompiler = manager.createCompiler();
          activeCompiler.outputFileSystem = fs;
        }

        // Populate one graph, then restore its build results into the other graph.
        const index = path.join(root, "src/index.js");
        write(index, executionFirst ? mainEntry : executorEntry);
        await run(activeCompiler, [index]);
        expect(readOutput(root).default).toBe("initial");
        expect(builds()).toBe(2);

      },
    })),
  ),
);
