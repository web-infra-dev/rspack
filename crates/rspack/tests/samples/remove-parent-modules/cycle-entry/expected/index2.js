(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["index2"], {
"./index2.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _shared__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./shared */"./shared.js");
/* harmony import */var _shared__WEBPACK_IMPORTED_MODULE_1__default = /*#__PURE__*/__webpack_require__.n(_shared__WEBPACK_IMPORTED_MODULE_1_);
__webpack_require__.el(/* ./index */"./index.js").then(__webpack_require__.bind(__webpack_require__, /* ./index */"./index.js"));

console.log('index2');
},
"./shared.js": function (module, exports, __webpack_require__) {
console.log('shared');
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index2.js"));

}
]);