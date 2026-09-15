const fs = require("fs");
const path = require("path");

module.exports = {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.join(options.output.path, "main.mjs"),
      "utf-8",
    );

    expect(source).not.toContain("export { __rspack_");
  },
};
