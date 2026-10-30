import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

let secondBuild;
let runtime;

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
  description: "should update dynamic entry imports and options on rebuild",
  options(testContext) {
    secondBuild = false;
    runtime = undefined;
    const fixture = testContext.getDist("fixture");
    fs.mkdirSync(fixture, { recursive: true });
    fs.writeFileSync(path.join(fixture, "package.json"), '{"sideEffects":false}');
    fs.writeFileSync(
      path.join(fixture, "entry.js"),
      'import { value } from "./barrel.js"; export const result = value;'
    );
    fs.writeFileSync(
      path.join(fixture, "barrel.js"),
      'export { value } from "./leaf.js";'
    );
    fs.writeFileSync(path.join(fixture, "leaf.js"), 'export const value = "before";');
    return {
      context: fs.realpathSync(fixture),
      mode: "development",
      target: "node",
      devtool: false,
      cache: false,
      entry: () => ({
        main: {
          import: secondBuild ? ["./entry.js", "./leaf.js"] : ["./entry.js"],
          ...(runtime ? { runtime } : {})
        }
      }),
      output: {
        path: testContext.getDist("output"),
        filename: "[name].cjs",
        library: { type: "commonjs2" }
      },
      optimization: { sideEffects: "flag", usedExports: false }
    };
  },
  compiler(context, compiler) {
    compiler.outputFileSystem = fs;
  },
  async build(context, compiler) {
    const build = options =>
      new Promise((resolve, reject) => {
        compiler.run((error, stats) => {
          if (error) return reject(error);
          if (stats.hasErrors()) return reject(new Error(stats.toString()));
          resolve(stats);
        }, options);
      });
    const readResult = () =>
      JSON.parse(
        execFileSync(
          process.execPath,
          ["-e", `console.log(JSON.stringify(require(${JSON.stringify(path.join(compiler.outputPath, "main.cjs"))})))`],
          { encoding: "utf8" }
        )
      );

    await build();
    expect(readResult()).toEqual({ result: "before" });

    secondBuild = true;
    const leaf = path.join(compiler.context, "leaf.js");
    fs.writeFileSync(leaf, 'export const value = "after";');
    compiler.inputFileSystem.purge?.();
    await build({ modifiedFiles: new Set([leaf]) });
    expect(readResult()).toEqual({ value: "after" });

    runtime = "other";
    await build({ modifiedFiles: new Set() });
    expect(readResult()).toEqual({ value: "after" });
    expect(fs.existsSync(path.join(compiler.outputPath, "other.cjs"))).toBe(true);
  }
};
