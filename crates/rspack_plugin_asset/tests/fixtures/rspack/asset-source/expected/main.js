(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _data_txt__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./data.txt */"./data.txt");

console.log(_data_txt__WEBPACK_IMPORTED_MODULE__);
},
"./data.txt": function (module, exports, __webpack_require__) {
module.exports = "- Isn't Rspack a gamechanging bundler?\n  - Hella yeah!";},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);