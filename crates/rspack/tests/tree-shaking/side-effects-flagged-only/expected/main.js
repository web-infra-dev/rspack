(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'something': function() { return _lib__WEBPACK_IMPORTED_MODULE_0_["default"]; }});
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./lib */"./lib.js");
/* harmony import */var _src_a__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./src/a */"./src/a.js");


},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _app__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./app */"./app.js");

(0, _app__WEBPACK_IMPORTED_MODULE_0_["something"])();
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
"./src/a.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
var __WEBPACK_DEFAULT_EXPORT__ = (()=>{
    console.log("");
});
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);