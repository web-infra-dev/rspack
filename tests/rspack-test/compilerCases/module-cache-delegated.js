const fs = require("node:fs");
const path = require("node:path");
const { DllReferencePlugin } = require("@rspack/core");

const dllName = "recovery-dll";
const sourceRequest = `dll-reference ${dllName}`;
const content = {
  "./numeric.js": {
    id: 7,
    exports: ["default", "unused", "value"],
    buildMeta: { exportsType: "namespace", strictEsmModule: true, esm: true },
  },
  "./named.js": {
    id: "named-export",
    exports: ["default", "value"],
    buildMeta: { exportsType: "namespace", strictEsmModule: true, esm: true },
  },
};
const identifier = (id) => `delegated ${id} from ${sourceRequest}`;
const expectedIdentifiers = Object.values(content)
  .map((entry) => identifier(entry.id))
  .sort();

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = ["persistent", "disabled"].map((cacheMode) => {
  let built = [];
  let reused = [];
  let firstRuntimeIds;

  return {
    description:
      cacheMode === "persistent"
        ? "should recover DelegatedModule dependencies, exports and code generation from persistent cache"
        : "should rebuild DelegatedModule when persistent caching is disabled",
    options(context) {
      const root = context.getDist(`delegated-${cacheMode}`);
      fs.rmSync(root, { recursive: true, force: true });
      fs.mkdirSync(root, { recursive: true });
      const entry = path.join(root, "index.js");
      fs.writeFileSync(
        entry,
        `
          import numericDefault, { value as numeric } from "dll/numeric";
          import namedDefault, { value as named } from "dll/named";
          export default [numericDefault, namedDefault, numeric, named];
        `,
      );
      const timestamp = new Date(Date.now() - 20000);
      fs.utimesSync(entry, timestamp, timestamp);
      return {
        context: root,
        entry: "./index.js",
        target: "node",
        mode: "production",
        devtool: false,
        incremental: false,
        cache:
          cacheMode === "persistent"
            ? {
                type: "persistent",
                storage: {
                  type: "filesystem",
                  location: path.join(root, "cache"),
                },
              }
            : false,
        experiments: {
          newCache: {
            module: true,
            loader: false,
            codeGeneration: false,
            devtool: false,
            minimize: false,
          },
        },
        output: {
          path: path.join(root, "dist"),
          filename: "main.js",
          library: { type: "commonjs2" },
        },
        optimization: {
          concatenateModules: false,
          providedExports: true,
          minimize: false,
        },
        plugins: [
          new DllReferencePlugin({
            scope: "dll",
            sourceType: "commonjs2",
            manifest: { name: dllName, content },
          }),
          {
            apply(compiler) {
              compiler.hooks.compilation.tap(
                "DelegatedModuleCache",
                (compilation) => {
                  compilation.hooks.buildModule.tap(
                    "DelegatedModuleCache",
                    (module) => {
                      if (module.identifier().startsWith("delegated ")) {
                        built.push(module.identifier());
                      }
                    },
                  );
                  compilation.hooks.stillValidModule.tap(
                    "DelegatedModuleCache",
                    (module) => {
                      if (module.identifier().startsWith("delegated ")) {
                        reused.push(module.identifier());
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
    async build(context) {
      const manager = context.getCompiler();
      for (let run = 0; run < 2; run++) {
        built = [];
        reused = [];
        if (run > 0) {
          // Closing flushes the persistent cache; a fresh compiler has no
          // previous module graph or in-memory module cache to recover from.
          await manager.close();
          manager.createCompiler();
        }

        const stats = await manager.build();
        expect(
          stats.toJson({ all: false, errors: true, warnings: true }),
        ).toMatchObject({
          errors: [],
          warnings: [],
        });
        const cacheHit = cacheMode === "persistent" && run > 0;
        expect(built.sort()).toEqual(cacheHit ? [] : expectedIdentifiers);
        expect(reused.sort()).toEqual(cacheHit ? expectedIdentifiers : []);

        const { compilation } = stats;
        const modules = [...compilation.modules].filter((module) =>
          module.identifier().startsWith("delegated "),
        );
        expect(modules.map((module) => module.identifier()).sort()).toEqual(
          expectedIdentifiers,
        );
        for (const module of modules) {
          const manifestEntry = Object.values(content).find(
            (entry) => identifier(entry.id) === module.identifier(),
          );
          expect(
            module.dependencies.map((dependency) => dependency.type),
          ).toEqual(["delegated source", "static exports"]);
          expect(module.dependencies[0].request).toBe(sourceRequest);
          const sourceModule = compilation.moduleGraph.getModule(
            module.dependencies[0],
          );
          expect(sourceModule?.identifier()).toMatch(/^external /);
          expect(
            compilation.moduleGraph.getProvidedExports(module).sort(),
          ).toEqual([...manifestEntry.exports].sort());
        }

        const runtimeIds = [];
        const dllExports = {
          7: {
            default: "numeric-default",
            value: "numeric-value",
            unused: true,
          },
          "named-export": { default: "named-default", value: "named-value" },
        };
        const output = { exports: {} };
        new Function(
          "require",
          "module",
          "exports",
          compilation.getAsset("main.js").source.source(),
        )(
          (request) => {
            expect(request).toBe(dllName);
            return (id) => {
              runtimeIds.push(id);
              return dllExports[id];
            };
          },
          output,
          output.exports,
        );
        // The DLL exports deliberately have no __esModule marker. Default imports
        // therefore verify that the cached namespace BuildMeta was restored.
        expect(output.exports.default).toEqual([
          "numeric-default",
          "named-default",
          "numeric-value",
          "named-value",
        ]);
        expect(runtimeIds.map(String).sort()).toEqual(["7", "named-export"]);
        if (run === 0) firstRuntimeIds = runtimeIds;
        else expect(runtimeIds).toEqual(firstRuntimeIds);
      }
    },
  };
});
