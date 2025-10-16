"use strict";
self["webpackHotUpdate"]("main", {
"./style.module.css": 
/*!**************************!*\
  !*** ./style.module.css ***!
  \**************************/
(function (module, __unused_webpack_exports, __webpack_require__) {
var exports = {
  "class-other": "_style_module_css-class-other",
};
// only invalidate when locals change
var stringified_exports = JSON.stringify(exports);
if (module.hot.data && module.hot.data.exports && module.hot.data.exports != stringified_exports) {
  module.hot.invalidate();
} else {
  module.hot.accept(); 
}
module.hot.dispose(function(data) { data.exports = stringified_exports; });
__webpack_require__.r(module.exports = exports);


}),

},function(__webpack_require__) {
// webpack/runtime/get_full_hash
(() => {
__webpack_require__.h = () => ("ab1971d47911cc4f")
})();

}
);