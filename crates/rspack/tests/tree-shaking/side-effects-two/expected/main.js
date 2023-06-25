(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'something': function() { return _lib__WEBPACK_IMPORTED_MODULE__["default"]; }});
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./lib */"./lib.js");

 // export {
 //   result as test
 // }
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _app__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./app */"./app.js");

(0, _app__WEBPACK_IMPORTED_MODULE__["something"])();
},
"./lib.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }});
 const secret = "888";
 const result = 20000;
 const something = function() {};
function __WEBPACK_DEFAULT_EXPORT__() {}
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);