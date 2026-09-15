const { execFile } = require("node:child_process");
const path = require("node:path");
const { promisify } = require("node:util");

const runNode = promisify(execFile);
const entries = {
  module: `
    import "@rspack/test-tools/setup-env";
    import { createNormalCase, describeByWalk } from "@rspack/test-tools";
    import { urlToRelativePath } from "@rspack/test-tools/helper/legacy/urlToRelativePath";
    import { StreamedEventReporter } from "@rspack/test-tools/reporter";
  `,
  commonjs: `
    require("@rspack/test-tools/setup-env");
    const { createNormalCase, describeByWalk } = require("@rspack/test-tools");
    const { urlToRelativePath } = require("@rspack/test-tools/helper/legacy/urlToRelativePath");
    const { StreamedEventReporter } = require("@rspack/test-tools/reporter");
  `,
};

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = Object.entries(entries).map(([format, imports]) => ({
  description: `should load test tools from ${format} without experimental VM modules`,
  async build() {
    const { stdout } = await runNode(
      process.execPath,
      [
        `--input-type=${format}`,
        "--eval",
        `${imports}
        console.log(JSON.stringify([
          typeof createNormalCase,
          typeof describeByWalk,
          typeof StreamedEventReporter,
          urlToRelativePath("https://example.com/bundle.js"),
        ]));`,
      ],
      {
        cwd: path.resolve(__dirname, ".."),
        env: { ...process.env, NODE_OPTIONS: "", RSTEST: "true" },
      },
    );
    expect(JSON.parse(stdout)).toEqual([
      "function",
      "function",
      "function",
      "./bundle.js",
    ]);
  },
}));
