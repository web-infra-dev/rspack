const fs = require("fs");
const path = require("path");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.resolve(options.output.path, "main.js"),
      "utf-8",
    );

    expect(source).toContain("__nested_rspack_module__");
    expect(source).toContain(
      'const __nested_rspack_module__ = "user rspack module";',
    );
    expect(source).toContain("const moduleId = __rspack_module.id;");
  },
};
