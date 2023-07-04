(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["index"], {
"./i-1.js": function (module, exports, __webpack_require__) {
console.log('i-1');
},
"./index.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _shared__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./shared */"./shared.js");
/* harmony import */var _shared__WEBPACK_IMPORTED_MODULE_1__default = /*#__PURE__*/__webpack_require__.n(_shared__WEBPACK_IMPORTED_MODULE_1_);
/* harmony import */var _i_1__WEBPACK_IMPORTED_MODULE_2_ = __webpack_require__(/* ./i-1 */"./i-1.js");
/* harmony import */var _i_1__WEBPACK_IMPORTED_MODULE_2__default = /*#__PURE__*/__webpack_require__.n(_i_1__WEBPACK_IMPORTED_MODULE_2_);


__webpack_require__.el(/* ./a */"./a.js").then(__webpack_require__.bind(__webpack_require__, /* ./a */"./a.js"));
console.log('index');
},
"./shared.js": function (module, exports, __webpack_require__) {
console.log('shared');
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);