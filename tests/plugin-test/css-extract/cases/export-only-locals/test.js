const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  const mainContent = fs.readFileSync(path.resolve(outputDirectory, "main.js"), "utf8");
  expect(mainContent).toContain(`__webpack_require__.d(__webpack_exports__, {
  aClass: () => (_1),
  bClass: () => (_2),
  cClass: () => (_3)
});
// extracted by css-extract-rspack-plugin
var _1 = "foo__style__a-class";
var _2 = "foo__style__b__class";
var _3 = "foo__style__cClass";`);
  expect(mainContent).toContain(`console.log({ aClass: _style_css__WEBPACK_IMPORTED_MODULE_0__.aClass, bClass: _style_css__WEBPACK_IMPORTED_MODULE_0__.bClass, cClass: _style_css__WEBPACK_IMPORTED_MODULE_0__.cClass })`);
};
