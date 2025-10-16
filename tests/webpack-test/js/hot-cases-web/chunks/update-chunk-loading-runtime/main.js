"use strict";
(self["webpackChunk"] = self["webpackChunk"] || []).push([["main"], {
"./index.js": 
/*!******************!*\
  !*** ./index.js ***!
  \******************/
(function (module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
/* ESM import */var vendor__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! vendor */ "./node_modules/vendor.js");

module.hot.data.ok = true;
module.hot.data.loadChunk = () => __webpack_require__.e(/*! import() */ "chunk_js").then(__webpack_require__.bind(__webpack_require__, /*! ./chunk */ "./chunk.js"));
module.hot.data.test = () => {
	expect(vendor__WEBPACK_IMPORTED_MODULE_0__["default"]).toBe(2);
};
module.hot.data.hash = __webpack_require__.h();


}),

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
__webpack_require__.O(0, ["vendors-node_modules_vendor_js",], function() {
        return __webpack_exec__("./index.js");
      });
var __webpack_exports__ = __webpack_require__.O();
module.exports = __webpack_exports__;

}
]);