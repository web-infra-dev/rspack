const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  const mainContent = fs.readFileSync(path.resolve(outputDirectory, "main.mjs"), "utf8");
  expect(mainContent).toContain(`__webpack_require__.d(__webpack_exports__, {
  "a-class": () => (_1),
  b__class: () => (_2),
  cClass: () => (_3)
});
// extracted by css-extract-rspack-plugin
var _1 = "DD6nw8MPcAMdtPEK";
var _2 = "W0R3gkykgVcMOZ0M";
var _3 = "qUfyjYvB1M0V8uYS";`);
  expect(mainContent).toContain(`console.log({ css: _style_css__WEBPACK_IMPORTED_MODULE_0__["default"], aClass: _style_css__WEBPACK_IMPORTED_MODULE_0__["a-class"], bClass: _style_css__WEBPACK_IMPORTED_MODULE_0__.b__class, cClass: _style_css__WEBPACK_IMPORTED_MODULE_0__.cClass })`);
};
