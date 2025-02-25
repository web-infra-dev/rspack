const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  const mainContent = fs.readFileSync(path.resolve(outputDirectory, "main.js"), "utf8");
  expect(mainContent).toContain(`__webpack_require__.d(__webpack_exports__, {
  cnA: () => (_1),
  cnB: () => (_2)
});
// extracted by css-extract-rspack-plugin
var _1 = () => "class-name-a";
var _2 = () => "class-name-b";`);
  expect(mainContent).toContain(`console.log((0,_style_css__WEBPACK_IMPORTED_MODULE_0__.cnA)(), (0,_style_css__WEBPACK_IMPORTED_MODULE_0__.cnB)());`);
};
