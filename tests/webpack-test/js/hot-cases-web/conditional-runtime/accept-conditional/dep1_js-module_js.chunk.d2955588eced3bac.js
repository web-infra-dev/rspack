"use strict";
(self["webpackChunk"] = self["webpackChunk"] || []).push([["dep1_js-module_js"], {
"./dep1.js": 
/*!*****************!*\
  !*** ./dep1.js ***!
  \*****************/
(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  "default": () => (__WEBPACK_DEFAULT_EXPORT__)
});
/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (43);


}),
"./module.js": 
/*!*******************!*\
  !*** ./module.js ***!
  \*******************/
(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
__webpack_require__.d(__webpack_exports__, {
  test: () => (test)
});
/* ESM import */var _shared__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./shared */ "./shared.js");


function test(next, done) {
	expect((0,_shared__WEBPACK_IMPORTED_MODULE_0__.f)()).toBe(42);
	next(() => {
		expect((0,_shared__WEBPACK_IMPORTED_MODULE_0__.f)()).toBe(43);
		done();
	});
}


}),

}]);