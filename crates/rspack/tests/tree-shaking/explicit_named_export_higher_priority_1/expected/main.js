(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./bar.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
 const a = 'bar';
},
"./foo.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'a': function() { return a; }});
/* harmony import */var _bar__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./bar */"./bar.js");
__webpack_require__.es(_bar__WEBPACK_IMPORTED_MODULE_0_, __webpack_exports__);
 const a = 'foo';

},
"./index.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _foo__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./foo */"./foo.js");

console.log(_foo__WEBPACK_IMPORTED_MODULE_0_["a"]);
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);