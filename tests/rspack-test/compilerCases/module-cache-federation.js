const fs = require("node:fs");
const path = require("node:path");
const { container } = require("@rspack/core");

const kinds = ["container entry", "remote", "fallback", "consume shared module", "provide shared module"];
const PLUGIN = "FederationModuleCacheTest";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
  { name: "memory" },
  { name: "persistent-reopen", reopen: true },
  { name: "enhanced-memory", enhanced: true },
  { name: "enhanced-persistent-reopen", enhanced: true, reopen: true }
].map(({ name, reopen, enhanced }) => {
  let root;
  let built;
  let succeeded;
  let stillValid;
  return {
    description: `should restore federation module builds with ${name}`,
    options(context) {
      root = context.getDist(name);
      fs.rmSync(root, { recursive: true, force: true });
      fs.mkdirSync(root, { recursive: true });
      const timestamp = new Date(Date.now() - 20000);
      for (const [file, source] of Object.entries({
        "index.js": 'import("remote/exposed"); import("direct/exposed"); import("./shared.js");',
        "exposed.js": 'module.exports = "exposed";',
        "shared.js": 'module.exports = "shared";'
      })) {
        const filename = path.join(root, file);
        fs.writeFileSync(filename, source);
        fs.utimesSync(filename, timestamp, timestamp);
      }
      const FederationPlugin = enhanced ? container.ModuleFederationPlugin : container.ModuleFederationPluginV1;
      return {
        context: root,
        entry: "./index.js",
        mode: "production",
        devtool: false,
        incremental: false,
        output: { path: path.join(root, "dist"), publicPath: "" },
        optimization: { runtimeChunk: "single" },
        cache: reopen ? {
          type: "persistent",
          storage: { type: "filesystem", location: path.join(root, "cache") }
        } : true,
        experiments: { newCache: { module: true, loader: false, codeGeneration: false, devtool: false, minimize: false } },
        plugins: [
          new FederationPlugin({
            name: "test_container",
            filename: "remoteEntry.js",
            exposes: { "./exposed": "./exposed.js" },
            remotes: {
              remote: ["remote@http://localhost/remote.js", "fallback@http://localhost/fallback.js"],
              direct: "direct@http://localhost/direct.js"
            },
            shared: { "./shared.js": { import: "./shared.js", requiredVersion: false, version: "1.0.0" } }
          }),
          {
            apply(compiler) {
              compiler.hooks.compilation.tap(PLUGIN, compilation => {
                const kindOf = module => kinds.find(kind => module.identifier().startsWith(`${kind} `));
                compilation.hooks.buildModule.tap(PLUGIN, module => {
                  const kind = kindOf(module);
                  if (kind) built.push(kind);
                });
                for (const hook of ["succeedModule", "stillValidModule"]) {
                  compilation.hooks[hook].tap(PLUGIN, module => {
                    const kind = kindOf(module);
                    if (kind) (hook === "succeedModule" ? succeeded : stillValid).push(kind);
                  });
                }
              });
            }
          }
        ]
      };
    },
    compiler(_, compiler) {
      compiler.outputFileSystem = fs;
    },
    async build(context) {
      const manager = context.getCompiler();
      let expectedOutput;
      for (let iteration = 0; iteration < 3; iteration++) {
        built = [];
        succeeded = [];
        stillValid = [];
        const stats = await manager.build();
        expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
        const expectedBuilt = iteration === 0 ? [...kinds].sort() : reopen ? ["consume shared module"] : [];
        expect([...new Set(built)].sort()).toEqual(expectedBuilt);
        expect([...new Set(succeeded)].sort()).toEqual(expectedBuilt);
        expect([...new Set(stillValid)].sort()).toEqual(kinds.filter(kind => !expectedBuilt.includes(kind)).sort());
        const output = Object.fromEntries(stats.compilation.getAssets()
          .filter(asset => asset.name.endsWith(".js"))
          .map(asset => [asset.name, asset.source.source().toString()]));
        if (enhanced) {
          for (const remote of ["remote", "fallback", "direct"]) {
            expect(output["runtime.js"]).toContain(`http://localhost/${remote}.js`);
          }
        }
        if (iteration === 0) expectedOutput = output;
        else expect(output).toEqual(expectedOutput);
        if (reopen && iteration < 2) {
          await manager.close();
          manager.createCompiler().outputFileSystem = fs;
        }
      }
    }
  };
});
