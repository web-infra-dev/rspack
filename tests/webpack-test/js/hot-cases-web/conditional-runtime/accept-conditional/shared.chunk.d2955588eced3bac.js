"use strict";
(self["webpackChunk"] = self["webpackChunk"] || []).push([["shared"], {
"./shared.js": 
/*!*******************!*\
  !*** ./shared.js ***!
  \*******************/
(function (module, __webpack_exports__, __webpack_require__) {
__webpack_require__.d(__webpack_exports__, {
  f: () => (f),
  g: () => (g)
});
if ("main" == __webpack_require__.j) {
  /* ESM import */var _dep1__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./dep1 */ "./dep1.js");

}if ("dep2_js-worker_js" == __webpack_require__.j) {
  /* ESM import */var _dep2__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./dep2 */ "./dep2.js");

}


function f() {
	return _dep1__WEBPACK_IMPORTED_MODULE_0__["default"];
}

function g() {
	return _dep2__WEBPACK_IMPORTED_MODULE_1__["default"];
}

module.hot.accept([/*! ./dep1 */ "./dep1.js", /*! ./dep2 */ "./dep2.js"], function(){
if ("main" == __webpack_require__.j) {
/* ESM import */_dep1__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./dep1 */ "./dep1.js");

}
if ("dep2_js-worker_js" == __webpack_require__.j) {
/* ESM import */_dep2__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./dep2 */ "./dep2.js");

}

});


}),

}]);