(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./b.js?x": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'a': function() { return a; }});
 const a = 3;
},
"./index.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _b_jsx__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./b.jsx */"./b.jsx");
/* harmony import */var _b_js_x__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./b.js?x */"./b.js?x");


_b_js_x__WEBPACK_IMPORTED_MODULE_1_["a"];
_b_jsx__WEBPACK_IMPORTED_MODULE_0_["a"];
console.log("hello, world");
},
"./b.jsx": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'a': function() { return a; }});
 const a = 3;
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);