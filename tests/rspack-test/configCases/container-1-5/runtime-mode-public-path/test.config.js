const fs = require("fs");
const path = require("path");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.resolve(options.output.path, "bundle0.js"),
      "utf-8",
    );

    expect(source).toContain("__rspack_context.p =");
    expect(source).not.toContain("__webpack_require__.p =");
  },
};
