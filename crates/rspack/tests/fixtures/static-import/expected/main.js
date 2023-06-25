(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _b__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./b */"./b.js");
/* harmony import */var _b__WEBPACK_IMPORTED_MODULE___default = /*#__PURE__*/__webpack_require__.n(_b__WEBPACK_IMPORTED_MODULE__);

console.log('a');
},
"./b.js": function (module, exports, __webpack_require__) {
console.log('b');
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _a__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./a */"./a.js");

console.log('hello, world');
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);