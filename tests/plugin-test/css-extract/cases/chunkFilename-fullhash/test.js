const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  const mainContent = fs.readFileSync(path.resolve(outputDirectory, "main.js"), "utf8");
  expect(mainContent).toContain("webpack/runtime/get mini-css chunk filename");
  expect(mainContent).toContain("__webpack_require__.miniCssF");
  expect(mainContent).toContain(`return "" + chunkId + ".$" + __webpack_require__.h() + "$.css"`);
};
