(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'app': function() { return app; }});
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./lib */"./lib.js");

 function app() {}
app.prototype.result = _lib__WEBPACK_IMPORTED_MODULE__["result"];
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _src_a__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./src/a */"./src/a.js");


},
"./lib.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'result': function() { return result; }});
 const secret = "888";
 const result = 20000;
 const something = function() {};
},
"./src/a.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var $_app__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ../app */"./app.js");

$_app__WEBPACK_IMPORTED_MODULE__["app"];
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);