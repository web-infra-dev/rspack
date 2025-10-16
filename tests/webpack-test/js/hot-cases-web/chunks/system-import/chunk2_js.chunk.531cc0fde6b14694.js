"use strict";
(self["webpackChunk"] = self["webpackChunk"] || []).push([["chunk2_js"], {
"./chunk2.js": 
/*!*******************!*\
  !*** ./chunk2.js ***!
  \*******************/
(function (module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  counter: () => (counter),
  value: () => (/* reexport safe */ _file__WEBPACK_IMPORTED_MODULE_0__.value),
  value2: () => (/* reexport safe */ _file2__WEBPACK_IMPORTED_MODULE_1__.value)
});
/* ESM import */var _file__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./file */ "./file.js");
/* ESM import */var _file2__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./file2 */ "./file2.js");


var counter = 0;
module.hot.accept(/*! ./file */ "./file.js", function(){
/* ESM import */_file__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./file */ "./file.js");

});
module.hot.accept(/*! ./file2 */ "./file2.js", function(__WEBPACK_OUTDATED_DEPENDENCIES__) {
/* ESM import */_file2__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./file2 */ "./file2.js");
(function() {
	counter++;
})(__WEBPACK_OUTDATED_DEPENDENCIES__); }.bind(this));


}),
"./file.js": 
/*!*****************!*\
  !*** ./file.js ***!
  \*****************/
(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  value: () => (value)
});
var value = 2;


}),
"./file2.js": 
/*!******************!*\
  !*** ./file2.js ***!
  \******************/
(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  value: () => (value)
});
var value = 4;


}),

}]);