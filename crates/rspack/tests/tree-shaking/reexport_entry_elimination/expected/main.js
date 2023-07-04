(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'b': function() { return _b_js__WEBPACK_IMPORTED_MODULE_0_["default"]; }});
/* harmony import */var _b_js__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./b.js */"./b.js");


},
"./b.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }});
/* harmony import */var _c_js__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./c.js */"./c.js");

var __WEBPACK_DEFAULT_EXPORT__ = 2000 + _c_js__WEBPACK_IMPORTED_MODULE_0_["default"];
},
"./c.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }});
var __WEBPACK_DEFAULT_EXPORT__ = 10;
},
"./index.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _a_js__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./a.js */"./a.js");

_a_js__WEBPACK_IMPORTED_MODULE_0_["b"];
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);