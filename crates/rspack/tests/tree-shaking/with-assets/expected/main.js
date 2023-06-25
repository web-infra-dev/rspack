(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'a': function() { return a; }});
 const a = 3;
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _a_js__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./a.js */"./a.js");
/* harmony import */var _a_svg__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./a.svg */"./a.svg");


_a_js__WEBPACK_IMPORTED_MODULE__["a"];
_a_svg__WEBPACK_IMPORTED_MODULE__;
},
"./a.svg": function (module, exports, __webpack_require__) {
module.exports = "data:image/svg+xml;base64,";},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);