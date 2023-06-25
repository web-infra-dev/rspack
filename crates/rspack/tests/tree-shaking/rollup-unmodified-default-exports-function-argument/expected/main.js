(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./foo.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }, 'bar': function() { return bar; }});
var foo = function() {
    return 42;
};
var __WEBPACK_DEFAULT_EXPORT__ = foo;
 function bar() {
    return contrivedExample(foo);
}
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _foo__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./foo */"./foo.js");

var answer = (0, _foo__WEBPACK_IMPORTED_MODULE__["default"])();
var somethingElse = (0, _foo__WEBPACK_IMPORTED_MODULE__["bar"])();
console.log(answer);
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);