(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./bar.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {});
function test() {}
},
"./foo.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {});
/* harmony import */var _bar__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./bar */"./bar.js");

},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _foo__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./foo */"./foo.js");

_foo__WEBPACK_IMPORTED_MODULE__["Select"]();
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);