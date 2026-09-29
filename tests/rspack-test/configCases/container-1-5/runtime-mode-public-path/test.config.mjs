import fs from "node:fs";
import path from "node:path";

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.resolve(options.output.path, "bundle0.js"),
      "utf-8",
    );

    expect(source).toContain("__rspack_context.p =");
    expect(source).toContain("var prevStartup = startup;");
    expect(source).not.toMatch(/\nvar startup = function/);
    expect(source).not.toContain("__webpack_require__.p =");
  },
};
