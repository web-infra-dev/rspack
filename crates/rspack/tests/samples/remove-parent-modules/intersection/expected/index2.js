(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["index2"], {
"./i-2.js": function (__unused_webpack_module, exports, __webpack_require__) {
console.log('i-2');
},
"./index2.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _shared__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./shared */"./shared.js");
/* harmony import */var _shared__WEBPACK_IMPORTED_MODULE_0__default = /*#__PURE__*/__webpack_require__.n(_shared__WEBPACK_IMPORTED_MODULE_0_);
/* harmony import */var _i_2__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./i-2 */"./i-2.js");
/* harmony import */var _i_2__WEBPACK_IMPORTED_MODULE_1__default = /*#__PURE__*/__webpack_require__.n(_i_2__WEBPACK_IMPORTED_MODULE_1_);


__webpack_require__.el(/* ./a */"./a.js").then(__webpack_require__.bind(__webpack_require__, /* ./a */"./a.js"));
console.log('index');
},
"./shared.js": function (__unused_webpack_module, exports, __webpack_require__) {
console.log('shared');
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index2.js"));

}
]);