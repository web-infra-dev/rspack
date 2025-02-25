const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  const mainContent = fs.readFileSync(path.resolve(outputDirectory, "runtime~main.js"), "utf8");
  expect(mainContent).toContain(`webpack/runtime/get mini-css chunk filename`);
};
