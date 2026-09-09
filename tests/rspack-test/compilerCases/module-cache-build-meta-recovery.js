const fs = require("node:fs");
const path = require("node:path");

function rebuild(compiler, modifiedFiles) {
  return new Promise((resolve, reject) => {
    compiler.__internal__rebuild(new Set(modifiedFiles), new Set(), error => {
      if (error) return reject(error);
      resolve(compiler._lastCompilation);
    });
  });
}

function expectAsync(compilation, expected) {
  for (const filename of ["index.js", "dep.js"]) {
    const module = [...compilation.modules].find(module =>
      module.resource?.endsWith(`/${filename}`)
    );
    expect(module).toBeDefined();
    expect(compilation.moduleGraph.isAsync(module)).toBe(expected);
  }
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description:
    "should preserve frozen build metadata across failed rebuilds and replace it on recovery",
  options(context) {
    const root = context.getDist("src");
    fs.mkdirSync(root, { recursive: true });
    fs.writeFileSync(
      path.join(root, "index.js"),
      'import { value } from "./dep.js"; console.log(value);'
    );
    fs.writeFileSync(
      path.join(root, "dep.js"),
      'export const value = await Promise.resolve("initial");'
    );
    return {
      context: root,
      entry: "./index.js",
      mode: "development",
      devtool: false,
      cache: true,
      incremental: true,
      experiments: {
        newCache: { module: true, loader: false, codeGeneration: false }
      },
      output: { path: context.getDist("output") },
      optimization: { concatenateModules: false }
    };
  },
  async build(context, compiler) {
    const initial = await new Promise((resolve, reject) => {
      compiler.run((error, stats) => {
        if (error) return reject(error);
        resolve(stats);
      });
    });
    expect(initial.hasErrors()).toBe(false);
    expectAsync(initial.compilation, true);

    const dependency = context.getDist("src/dep.js");
    fs.writeFileSync(dependency, "export const value = ;");
    const failed = await rebuild(compiler, [dependency]);
    expect(failed.errors).toHaveLength(1);
    expectAsync(failed, true);

    // Recovering metadata from another failed build must retain the last
    // successful Arc, even when that metadata already belongs to the cache.
    const failedAgain = await rebuild(compiler, [dependency]);
    expect(failedAgain.errors).toHaveLength(1);
    expectAsync(failedAgain, true);

    fs.writeFileSync(dependency, 'export const value = "recovered";');
    const recovered = await rebuild(compiler, [dependency]);
    expect(recovered.errors).toHaveLength(0);
    expectAsync(recovered, false);
  }
};
