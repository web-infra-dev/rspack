(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'a': function() { return a; },
  'b': function() { return b; }
});
 const a = 3;
 const b = 3;
},
"./answer.js": function (__unused_webpack_module, exports, __webpack_require__) {
const res = __webpack_require__(/* ./lib.js */"./lib.js");
exports.test = function() {
    res;
};
},
"./index.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _answer__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./answer */"./answer.js");
/* harmony import */var _answer__WEBPACK_IMPORTED_MODULE_0__default = /*#__PURE__*/__webpack_require__.n(_answer__WEBPACK_IMPORTED_MODULE_0_);

_answer__WEBPACK_IMPORTED_MODULE_0_.test;
},
"./lib.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _a_js__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./a.js */"./a.js");
__webpack_require__.es(_a_js__WEBPACK_IMPORTED_MODULE_0_, __webpack_exports__);

},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);