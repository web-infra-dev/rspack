"use strict";
(self["webpackChunk"] = self["webpackChunk"] || []).push([["chunk1_js"], {
"./chunk1.js": 
/*!*******************!*\
  !*** ./chunk1.js ***!
  \*******************/
(function (module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  active: () => (/* reexport safe */ _shared__WEBPACK_IMPORTED_MODULE_0__.active)
});
/* ESM import */var _shared__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./shared */ "./shared.js");

module.hot.accept(/*! ./shared */ "./shared.js", function(){
/* ESM import */_shared__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./shared */ "./shared.js");

});


}),
"./shared.js": 
/*!*******************!*\
  !*** ./shared.js ***!
  \*******************/
(function (module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  active: () => (active)
});
let active = true;

module.hot.dispose(() => {
	active = false;
});


}),

}]);