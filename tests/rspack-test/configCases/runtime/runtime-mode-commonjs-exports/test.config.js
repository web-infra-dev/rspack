const fs = require("fs");
const path = require("path");

module.exports = {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.resolve(options.output.path, "bundle0.js"),
      "utf-8",
    );

    expect(source).toContain("var __rspack_unused_export;");
    expect(source).toContain('__rspack_unused_export = "unused";');
    expect(source).toContain("__rspack_unused_export = ({");
    expect(source).not.toContain("__webpack_unused_export__");
    expect(source).not.toContain("__webpack_unused_export");
  },
};
