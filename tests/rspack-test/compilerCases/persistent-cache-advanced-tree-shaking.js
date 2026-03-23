const fs = require("node:fs");
const path = require("path");

const CASE_DIR = "persistent-cache-advanced-tree-shaking";
const CACHE_DIR = ".cache";
const OUTPUT_DIR = "output";
const WORK_DIR = "workdir";

function readOutput(context) {
  return fs.readFileSync(context.getDist(path.join(OUTPUT_DIR, "main.js")), "utf-8");
}

async function recreateCompiler(context) {
  const compilerManager = context.getCompiler();
  await compilerManager.close();
  const compiler = compilerManager.createCompiler();
  compiler.outputFileSystem = fs;
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description: "should recover pure modules after persistent cache rebuild",
  options(context) {
    const sourceDir = context.getSource(CASE_DIR);
    const workDir = context.getDist(WORK_DIR);
    fs.rmSync(workDir, { recursive: true, force: true });
    fs.cpSync(sourceDir, workDir, { recursive: true });

    return {
      mode: "production",
      target: "node",
      context: workDir,
      entry: "./index.js",
      experiments: {
        advancedTreeShaking: true,
        cache: {
          type: "persistent",
          buildDependencies: [__filename],
          storage: {
            type: "filesystem",
            directory: context.getDist(CACHE_DIR)
          }
        }
      },
      optimization: {
        sideEffects: true,
        innerGraph: true,
        usedExports: true,
        concatenateModules: false
      },
      output: {
        path: context.getDist(OUTPUT_DIR),
        filename: "main.js",
        clean: true
      }
    };
  },
  async compiler(_, compiler) {
    compiler.outputFileSystem = fs;
  },
  async build(context) {
    const compilerManager = context.getCompiler();
    await compilerManager.build();
    context.setValue("firstOutput", readOutput(context));

    const workDir = context.getDist(WORK_DIR);
    fs.copyFileSync(
      path.join(workDir, "dep.pure.js"),
      path.join(workDir, "dep.js")
    );

    await recreateCompiler(context);
    await compilerManager.build();
    context.setValue("secondOutput", readOutput(context));
  },
  async check({ context }) {
    const firstOutput = context.getValue("firstOutput");
    const secondOutput = context.getValue("secondOutput");

    expect(firstOutput).toContain("./bridge.js");
    expect(firstOutput).toContain("./dep.js");
    expect(firstOutput).toContain("./tracker.js");
    expect(firstOutput).toContain("impure");

    expect(secondOutput).not.toContain("./bridge.js");
    expect(secondOutput).not.toContain("./dep.js");
    expect(secondOutput).not.toContain("./tracker.js");
    expect(secondOutput).not.toContain("impure");
  }
};
