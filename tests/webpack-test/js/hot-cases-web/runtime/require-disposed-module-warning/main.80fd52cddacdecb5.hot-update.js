self["webpackHotUpdate"]("main", {
"./b.js": 
/*!**************!*\
  !*** ./b.js ***!
  \**************/
(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  "default": () => (__WEBPACK_DEFAULT_EXPORT__)
});
/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = ("b");


}),
"./module.js": 
/*!*******************!*\
  !*** ./module.js ***!
  \*******************/
(function (module, __unused_webpack_exports, __webpack_require__) {
module.exports = () => __webpack_require__(/*! ./b */ "./b.js");


}),

},function(__webpack_require__) {
// webpack/runtime/get_full_hash
(() => {
__webpack_require__.h = () => ("2229eac657aef5c3")
})();

}
);