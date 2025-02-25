const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  const mainContent = fs.readFileSync(path.resolve(outputDirectory, "main.js"), "utf8");
  expect(mainContent).not.toContain(`a-class`);
  expect(mainContent).toContain(`console.log({ aClass: _style_css__WEBPACK_IMPORTED_MODULE_0__.aClass, bClass: _style_css__WEBPACK_IMPORTED_MODULE_0__.bClass, cClass: _style_css__WEBPACK_IMPORTED_MODULE_0__.cClass })`);
};
