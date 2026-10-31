import fs from "node:fs";
import path from "node:path";

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
  description: "should keep code splitting errors across rebuilds",
  options(testContext) {
    const fixture = testContext.getDist("fixture");
    fs.mkdirSync(fixture, { recursive: true });
    for (const [file, source] of Object.entries({
      "package.json": '{"sideEffects":false}',
      "a.js": 'console.log("a");',
      "b.js": 'import { value } from "./barrel.js"; console.log(value);',
      "barrel.js": 'export { value } from "./leaf.js";',
      "leaf.js": 'export const value = "before";'
    })) fs.writeFileSync(path.join(fixture, file), source);
    return {
      context: fs.realpathSync(fixture),
      mode: "development",
      target: "node",
      devtool: false,
      cache: false,
      entry: {
        a: "./a.js",
        b: { import: "./b.js", dependOn: "a", runtime: "other" }
      },
      optimization: { sideEffects: "flag", usedExports: false }
    };
  },
  async build(context, compiler) {
    for (const [file, source] of [
      [null, null],
      ["leaf.js", 'export const value = "after";'],
      ["a.js", 'console.log("plain leaf edit");']
    ]) {
      const modifiedFiles = new Set();
      if (file) {
        const filename = path.join(compiler.context, file);
        fs.writeFileSync(filename, source);
        modifiedFiles.add(filename);
        compiler.inputFileSystem.purge?.();
      }
      const stats = await new Promise((resolve, reject) => {
        compiler.run((error, stats) => error ? reject(error) : resolve(stats), { modifiedFiles });
      });
      expect(stats.toJson({ all: false, errors: true }).errors).toEqual([
        expect.objectContaining({ message: expect.stringContaining("has 'dependOn' and 'runtime' specified") })
      ]);
    }
  }
};
