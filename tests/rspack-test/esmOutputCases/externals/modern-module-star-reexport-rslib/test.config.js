const fs = require("fs");
const path = require("path");

function readOutput(options) {
  return fs
    .readdirSync(options.output.path)
    .filter(file => file.endsWith(".mjs"))
    .map(file => fs.readFileSync(path.join(options.output.path, file), "utf-8"))
    .join("\n");
}

module.exports = {
  findBundle() {
    return [];
  },
  afterExecute(options) {
    const source = readOutput(options);

    expect(source).toContain('export * from "externals";');
    expect(source).not.toContain("__webpack_require__");
  },
};
