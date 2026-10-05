import path from "node:path";
import { createFsFromVolume, Volume } from "memfs";
import { Compilation, sources, SwcJsMinimizerRspackPlugin } from "@rspack/core";

const many = Array.from({ length: 256 }, (_, i) => `/*! @license package-${String(i).padStart(3, "0")} */`);
const cases = [
  { name: "no matching comments", comments: ["/* ordinary */"], expected: undefined },
  { name: "duplicate block and line comments with Unicode", comments: ["/*! @license 中文 */", "// @license same", "/* @license same*/", "/*! @license 中文 */", "// @license same"], expected: "/* @license same*/\n\n/*! @license 中文 */\n\n// @license same" },
  { name: "many distinct and repeated comments", comments: [...many.toReversed(), ...many], expected: many.join("\n\n") }
];

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default cases.map(scenario => {
  let outputFs;
  let outputPath;
  return {
    description: `should preserve extracted comment text and order for ${scenario.name}`,
    options(context) {
      outputPath = context.getDist();
      return {
        mode: "production",
        entry: "./a",
        output: { path: outputPath, filename: "main.js" },
        optimization: { minimize: true, minimizer: [new SwcJsMinimizerRspackPlugin({ extractComments: true })] },
        plugins: [{ apply(compiler) {
          compiler.hooks.thisCompilation.tap("CommentDedup", compilation => {
            compilation.hooks.processAssets.tap({ name: "CommentDedup", stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL }, () => {
              const code = scenario.comments.map((comment, i) => `${comment}\nglobalThis.commentProbe(${i});`).join("\n");
              compilation.emitAsset("comments.js", new sources.RawSource(code));
            });
          });
        } }]
      };
    },
    compiler(context, compiler) {
      outputFs = createFsFromVolume(new Volume());
      compiler.outputFileSystem = outputFs;
    },
    check() {
      const licensePath = path.join(outputPath, "comments.js.LICENSE.txt");
      expect(outputFs.existsSync(licensePath)).toBe(scenario.expected !== undefined);
      if (scenario.expected !== undefined) {
        expect(outputFs.readFileSync(licensePath, "utf8")).toBe(scenario.expected);
        expect(outputFs.readFileSync(path.join(outputPath, "comments.js"), "utf8").includes("LICENSE: comments.js.LICENSE.txt")).toBe(true);
      }
    }
  };
});
