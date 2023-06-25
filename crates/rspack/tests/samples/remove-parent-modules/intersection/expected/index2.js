(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["index2"], {
"./i-2.js": function (module, exports, __webpack_require__) {
console.log('i-2');
},
"./index2.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _shared__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./shared */"./shared.js");
/* harmony import */var _shared__WEBPACK_IMPORTED_MODULE___default = /*#__PURE__*/__webpack_require__.n(_shared__WEBPACK_IMPORTED_MODULE__);
/* harmony import */var _i_2__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./i-2 */"./i-2.js");
/* harmony import */var _i_2__WEBPACK_IMPORTED_MODULE___default = /*#__PURE__*/__webpack_require__.n(_i_2__WEBPACK_IMPORTED_MODULE__);


__webpack_require__.el(/* ./a */"./a.js").then(__webpack_require__.bind(__webpack_require__, /* ./a */"./a.js"));
console.log('index');
},
"./shared.js": function (module, exports, __webpack_require__) {
console.log('shared');
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index2.js"));

}
]);