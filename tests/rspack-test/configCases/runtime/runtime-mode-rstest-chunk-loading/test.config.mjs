import fs from "node:fs";
import path from "node:path";

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.resolve(options.output.path, "main.js"),
      "utf-8",
    );

    expect(source).toContain("__rspack_require.rstest_original_modules || {}");
    expect(source).not.toContain("__webpack_require__.rstest_original_modules");
  },
};
