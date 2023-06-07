(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./lib */"./lib.js");
/* harmony import */var _src_a__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./src/a */"./src/a.js");
__webpack_require__.es(_lib__WEBPACK_IMPORTED_MODULE__, exports);
__webpack_require__.es(_src_a__WEBPACK_IMPORTED_MODULE__, exports);

 // export {
 //   result as test
 // }
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _app__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./app */"./app.js");

_app__WEBPACK_IMPORTED_MODULE__["something"](); // a;
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);