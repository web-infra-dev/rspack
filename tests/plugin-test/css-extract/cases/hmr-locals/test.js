const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  const mainContent = fs.readFileSync(path.resolve(outputDirectory, "main.js"), "utf8");
  expect(mainContent).toContain("webpack/runtime/css loading");
  expect(mainContent).toContain("__webpack_require__.hmrC.miniCss");
  expect(mainContent).toContain("__webpack_require__.miniCssF");
  expect(mainContent).toContain(`var localsJsonString = "{\\"x\\":\\"uKUN7__BVSobrm9K\\"}"`);
};
