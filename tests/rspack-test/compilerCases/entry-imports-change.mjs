import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

let secondBuild;
let runtime;
let include;

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
  description: "should update dynamic entry imports and options on rebuild",
  options(testContext) {
    secondBuild = false;
    runtime = undefined;
    include = false;
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
    fs.writeFileSync(path.join(fixture, "included.js"), 'globalThis.included = "included";');
    fs.writeFileSync(path.join(fixture, "leaf.js"), 'export const value = "before";');
    return {
      context: fs.realpathSync(fixture),
      mode: "development",
      target: "node",
      devtool: false,
      cache: false,
      entry: () => ({
        keep: "./included.js",
        main: {
          import: secondBuild ? ["./entry.js", "./leaf.js"] : ["./entry.js"],
          ...(runtime ? { runtime } : {})
        }
      }),
      plugins: [{
        apply(compiler) {
          compiler.hooks.finishMake.tapPromise("IncludeOnRebuild", async compilation => {
            if (!include) return;
            await new Promise((resolve, reject) => compilation.addInclude(
              compiler.context,
              compiler.rspack.EntryPlugin.createDependency("./included.js"),
              { name: "main" },
              error => error ? reject(error) : resolve()
            ));
          });
        }
      }],
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

    const entry = path.join(compiler.context, "entry.js");
    fs.writeFileSync(entry, `
      import { value } from "./barrel.js";
      try { __webpack_require__(require.resolveWeak("./included.js")); } catch {}
      export const result = value + ":" + (globalThis.included || "absent");
    `);
    secondBuild = false;
    await build({ modifiedFiles: new Set([entry]) });
    expect(readResult()).toEqual({ result: "after:absent" });

    include = true;
    await build({ modifiedFiles: new Set() });
    expect(readResult()).toEqual({ result: "after:included" });

    include = false;
    await build({ modifiedFiles: new Set() });
    expect(readResult()).toEqual({ result: "after:absent" });
  }
};
