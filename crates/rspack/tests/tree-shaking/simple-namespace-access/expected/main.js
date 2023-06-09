(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _maths_js__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./maths.js */"./maths.js");
/* TREE-SHAKING */ 
console.log(_maths_js__WEBPACK_IMPORTED_MODULE__.xxx.test);
console.log(_maths_js__WEBPACK_IMPORTED_MODULE__['square']);
},
"./maths.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'xxx': function() { return _test_js__WEBPACK_IMPORTED_MODULE__; }});
__webpack_require__.d(exports, {'square': function() { return square; }});
/* harmony import */var _test_js__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./test.js */"./test.js");
// maths.js
// This function isn't used anywhere, so
// Rollup excludes it from the bundle...
 function square(x) {
    return x * x;
}
// This function gets included
 function cube(x) {
    // rewrite this as `square( x ) * x`
    // and see what happens!
    return x * x * x;
}

},
"./test.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'test': function() { return test; }, 'ccc': function() { return ccc; }});
 function test() {}
 function ccc() {}
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);