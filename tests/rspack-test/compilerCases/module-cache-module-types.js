const fs = require("node:fs");
const path = require("node:path");
const { createRequire } = require("node:module");
const { CssExtractRspackPlugin } = require("@rspack/core");
const { RawSource } = require("webpack-sources");

const PLUGIN = "ModuleCacheTypesTest";
const kinds = ["external", "raw", "sync", "lazy", "css", "json"];

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
  { name: "memory", cache: true },
  { name: "persistent", cache: "persistent" },
  { name: "persistent-without-memory", cache: "persistent", maxMemoryGenerations: 0 },
  { name: "persistent-reopen", cache: "persistent", reopen: true },
  { name: "disabled", cache: false }
].map(({ name, cache, maxMemoryGenerations, reopen }) => {
  let root;
  let iteration;
  let built;
  let succeeded;
  let timestamp;
  const assets = new Map(kinds.map(kind => [kind, []]));

  function write(relative, content) {
    const file = path.join(root, relative);
    fs.writeFileSync(file, content);
    fs.utimesSync(file, timestamp, timestamp);
  }

  function kindOf(module) {
    const identifier = module.identifier();
    if (identifier.startsWith("external ")) return "external";
    if (identifier.startsWith("ignored|")) return "raw";
    if (identifier.startsWith(`${path.join(root, "items")}|sync`)) return "sync";
    if (identifier.startsWith(`${path.join(root, "items")}|lazy`)) return "lazy";
    if (module.type === "css/mini-extract") return "css";
    if (module.resource === path.join(root, "data.json")) return "json";
  }

  return {
    description: `should cache module types and skip unsupported serialization with ${name}`,
    options(context) {
      root = context.getDist(name);
      fs.rmSync(root, { recursive: true, force: true });
      fs.mkdirSync(path.join(root, "items"), { recursive: true });
      timestamp = new Date(Date.now() - 20000);
      write("index.js", `
        require("./style.css");
        const sync = require.context("./items", false, /\\.js$/);
        const lazy = require.context("./items", false, /\\.js$/, "lazy");
        module.exports = async () => ({
          sync: sync.keys().sort().map(key => sync(key)),
          lazy: await Promise.all(lazy.keys().sort().map(key => lazy(key))),
          external: require("external").join("a", "b"),
          ignored: require("ignored"),
          json: require("./data.json")
        });
      `);
      write("items/a.js", 'module.exports = "a";');
      write("style.css", ".example { color: red; }");
      write("data.json", '{"value": 42}');
      fs.utimesSync(path.join(root, "items"), timestamp, timestamp);
      return {
        context: root,
        entry: "./index.js",
        target: "node",
        mode: "production",
        devtool: false,
        incremental: false,
        output: {
          path: path.join(root, "dist"),
          filename: "main.js",
          publicPath: "",
          library: { type: "commonjs2" }
        },
        cache: cache === "persistent" ? {
          type: "persistent",
          ...(maxMemoryGenerations === undefined ? {} : { maxMemoryGenerations }),
          storage: { type: "filesystem", location: path.join(root, "cache") }
        } : cache,
        experiments: {
          css: false,
          newCache: {
            module: true,
            loader: false,
            codeGeneration: false,
            devtool: false,
            minimize: false
          }
        },
        resolve: { alias: { ignored: false } },
        externals: { external: "commonjs node:path" },
        module: {
          rules: [
            { test: /\.css$/, type: "javascript/auto", use: [CssExtractRspackPlugin.loader, require.resolve("css-loader")] },
            // A function in parser options is deliberately unsupported by serialization.
            { test: /\.json$/, type: "json", parser: { parse: JSON.parse } }
          ]
        },
        plugins: [new CssExtractRspackPlugin({ filename: "main.css" }), {
          apply(compiler) {
            compiler.hooks.compilation.tap(PLUGIN, compilation => {
              compilation.hooks.buildModule.tap(PLUGIN, module => {
                const kind = kindOf(module);
                if (kind) built.push(kind);
              });
              compilation.hooks.succeedModule.tap(PLUGIN, module => {
                const kind = kindOf(module);
                if (!kind) return;
                succeeded.push(kind);
                if (built.includes(kind)) assets.set(kind, []);
                expect(Object.keys(module.buildInfo.assets).sort()).toEqual(assets.get(kind));
                const asset = `${kind}-${iteration}.txt`;
                module.emitFile(asset, new RawSource(asset));
                assets.get(kind).push(asset);
              });
            });
          }
        }]
      };
    },
    compiler(_, compiler) {
      compiler.outputFileSystem = fs;
    },
    async build(context) {
      const manager = context.getCompiler();
      const memory = cache !== false && maxMemoryGenerations !== 0 && !reopen;
      for (iteration = 0; iteration < 5; iteration++) {
        built = [];
        succeeded = [];
        timestamp = new Date(timestamp.getTime() + 1000);
        if (iteration === 2) {
          write("items/b.js", 'module.exports = "b";');
          write("style.css", ".example { color: blue; }");
        }
        if (iteration === 4) fs.unlinkSync(path.join(root, "items/a.js"));
        if (iteration === 2 || iteration === 4) {
          fs.utimesSync(path.join(root, "items"), timestamp, timestamp);
        }
        const stats = await manager.build();
        expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
        expect(succeeded.sort()).toEqual([...kinds].sort());
        const expected = kinds.filter(kind => {
          if (iteration === 0 || cache === false) return true;
          if (kind === "sync" || kind === "lazy") return !memory || iteration === 2 || iteration === 4;
          if (kind === "json") return !memory;
          return kind === "css" && iteration === 2;
        });
        expect(built.sort()).toEqual(expected.sort());
        const filename = path.join(root, "dist/main.js");
        const output = { exports: {} };
        new Function("require", "module", "exports", fs.readFileSync(filename, "utf8"))(
          createRequire(filename), output, output.exports
        );
        const values = iteration < 2 ? ["a"] : iteration < 4 ? ["a", "b"] : ["b"];
        expect(await output.exports()).toEqual({
          sync: values, lazy: values, external: path.join("a", "b"), ignored: {}, json: { value: 42 }
        });
        expect(fs.readFileSync(path.join(root, "dist/main.css"), "utf8"))
          .toContain(iteration < 2 ? "color: red" : "color: blue");
        if (reopen && iteration < 4) {
          await manager.close();
          manager.createCompiler().outputFileSystem = fs;
        }
      }
    }
  };
});
