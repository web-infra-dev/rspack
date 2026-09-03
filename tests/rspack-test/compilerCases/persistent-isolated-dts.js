const fs = require("node:fs");
const path = require("node:path");
const {
  experiments: { RslibPlugin }
} = require("@rspack/core");

const CASE_DIR = "persistent-isolated-dts";

async function buildAndCountLoaderRuns(context, counterFile) {
  const before = fs.existsSync(counterFile)
    ? fs.readFileSync(counterFile, "utf-8").length
    : 0;
  const stats = await context.getCompiler().build();
  expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
  return fs.readFileSync(counterFile, "utf-8").length - before;
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
  {
    name: "legacy-cache",
    newCache: false,
    warmLoaderRuns: 0
  },
  {
    name: "new-cache-without-loader-cache",
    newCache: { loader: false },
    warmLoaderRuns: 1
  },
  {
    name: "loader-cache-only",
    newCache: {
      codeGeneration: false,
      devtool: false,
      loader: true,
      minimize: false
    },
    warmLoaderRuns: 0
  }
].map(({ name, newCache, warmLoaderRuns }) => {
  let root;
  let dtsPath;
  let counterFile;

  function configureCompiler(compiler) {
    compiler.outputFileSystem = fs;
    expect(compiler.options.cache.type).toBe("persistent");
    if (newCache === false) {
      expect(compiler.options.experiments.newCache).toBe(false);
    } else {
      expect(compiler.options.experiments.newCache).toMatchObject(newCache);
    }
  }

  return {
    description: `should emit isolated declarations across persistent builds with ${name}`,
    options(context) {
      root = context.getDist(name);
      const sourceDir = path.resolve(__dirname, "../fixtures", CASE_DIR);
      const workDir = path.join(root, "workdir");
      dtsPath = path.join(workDir, "dist/types/index.d.ts");
      counterFile = path.join(root, "loader-runs.txt");
      fs.rmSync(root, { recursive: true, force: true });
      fs.cpSync(sourceDir, workDir, { recursive: true });

      return {
        context: workDir,
        mode: "production",
        entry: "./index.ts",
        target: "node",
        incremental: false,
        output: {
          path: workDir,
          filename: "output/main.js",
          library: {
            type: "commonjs"
          }
        },
        cache: {
          type: "persistent",
          buildDependencies: [__filename],
          storage: {
            type: "filesystem",
            location: path.join(root, ".cache")
          }
        },
        experiments: {
          newCache
        },
        module: {
          rules: [
            {
              test: /\.ts$/,
              type: "javascript/auto",
              use: [
                {
                  loader: "builtin:swc-loader",
                  cache: true,
                  options: {
                    jsc: {
                      parser: {
                        syntax: "typescript"
                      },
                      experimental: {
                        emitIsolatedDts: true
                      }
                    }
                  }
                },
                {
                  loader: path.join(workDir, "count-loader.js"),
                  cache: true,
                  options: { counterFile }
                }
              ]
            }
          ]
        },
        plugins: [
          new RslibPlugin({
            emitDts: {
              rootDir: workDir,
              declarationDir: "./dist/types"
            }
          })
        ]
      };
    },
    compiler(_, compiler) {
      configureCompiler(compiler);
    },
    async build(context) {
      const compilerManager = context.getCompiler();
      expect(await buildAndCountLoaderRuns(context, counterFile)).toBe(1);
      context.setValue("firstOutput", fs.readFileSync(dtsPath, "utf-8"));

      // Flush persistent cache and remove the declaration so the next compiler
      // must emit it again, even when SWC does not run.
      await compilerManager.close();
      fs.rmSync(dtsPath);
      configureCompiler(compilerManager.createCompiler());

      expect(await buildAndCountLoaderRuns(context, counterFile)).toBe(
        warmLoaderRuns
      );
      context.setValue("secondOutput", fs.readFileSync(dtsPath, "utf-8"));
    },
    check({ context }) {
      const firstOutput = context.getValue("firstOutput");
      const secondOutput = context.getValue("secondOutput");

      expect(firstOutput).toContain("export interface Foo");
      expect(firstOutput).toContain("export declare const foo: Foo;");
      expect(secondOutput).toBe(firstOutput);
    }
  };
});
