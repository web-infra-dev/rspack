const fs = require("node:fs");
const path = require("node:path");
const {
  experiments: { RslibPlugin }
} = require("@rspack/core");

const CASE_DIR = "persistent-isolated-dts";
const CACHE_DIR = ".cache";
const OUTPUT_DIR = "output";
const WORK_DIR = "workdir";

function getDtsPath(context) {
  return context.getDist(path.join(WORK_DIR, "dist/types/index.d.ts"));
}

function readDts(context) {
  return fs.readFileSync(getDtsPath(context), "utf-8");
}

async function recreateCompiler(context) {
  const compilerManager = context.getCompiler();
  await compilerManager.close();
  const compiler = compilerManager.createCompiler();
  compiler.outputFileSystem = fs;
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description:
    "should re-emit declaration files assets after persistent cache recovery",
  options(context) {
    const sourceDir = path.resolve(__dirname, "../fixtures", CASE_DIR);
    const workDir = context.getDist(WORK_DIR);
    fs.rmSync(workDir, { recursive: true, force: true });
    fs.cpSync(sourceDir, workDir, { recursive: true });

    return {
      context: workDir,
      entry: "./index.ts",
      target: "node",
      output: {
        path: workDir,
        filename: `${OUTPUT_DIR}/main.js`,
        library: {
          type: "commonjs"
        }
      },
      experiments: {
        cache: {
          type: "persistent",
          buildDependencies: [__filename],
          storage: {
            type: "filesystem",
            directory: context.getDist(CACHE_DIR)
          }
        }
      },
      module: {
        rules: [
          {
            test: /\.ts$/,
            type: "javascript/auto",
            use: {
              loader: "builtin:swc-loader",
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
            }
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
  async compiler(_, compiler) {
    compiler.outputFileSystem = fs;
  },
  async build(context) {
    const compilerManager = context.getCompiler();
    await compilerManager.build();
    context.setValue("firstOutput", readDts(context));

    fs.rmSync(getDtsPath(context), { force: true });

    await recreateCompiler(context);
    await compilerManager.build();
    context.setValue("secondOutput", readDts(context));
  },
  async check({ context }) {
    const firstOutput = context.getValue("firstOutput");
    const secondOutput = context.getValue("secondOutput");

    expect(firstOutput).toContain("export interface Foo");
    expect(firstOutput).toContain("export declare const foo: Foo;");
    expect(secondOutput).toContain("export interface Foo");
    expect(secondOutput).toContain("export declare const foo: Foo;");
  }
};
