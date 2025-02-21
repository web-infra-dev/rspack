const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  expect(fs.existsSync(path.resolve(outputDirectory, "one-main.js"))).toBe(true);
  expect(fs.existsSync(path.resolve(outputDirectory, "two-main.js"))).toBe(true);
};
