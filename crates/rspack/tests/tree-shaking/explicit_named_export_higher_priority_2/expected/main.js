(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./bar.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
 const a = 'bar';
},
"./baz.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'a': function() { return a; }});
 const a = 'baz';
},
"./foo.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'a': function() { return _baz__WEBPACK_IMPORTED_MODULE_0_["a"]; }});
/* harmony import */var _baz__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./baz */"./baz.js");
/* harmony import */var _bar__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./bar */"./bar.js");
__webpack_require__.es(_bar__WEBPACK_IMPORTED_MODULE_1_, exports);


},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _foo__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./foo */"./foo.js");

console.log(_foo__WEBPACK_IMPORTED_MODULE_0_["a"]);
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);