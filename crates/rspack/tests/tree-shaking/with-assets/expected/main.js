(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'a': function() { return a; }});
 const a = 3;
},
"./index.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _a_js__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./a.js */"./a.js");
/* harmony import */var _a_svg__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./a.svg */"./a.svg");


_a_js__WEBPACK_IMPORTED_MODULE_0_["a"];
_a_svg__WEBPACK_IMPORTED_MODULE_1_;
},
"./a.svg": function (module, exports, __webpack_require__) {
module.exports = "data:image/svg+xml;base64,";},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);