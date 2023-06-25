(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'a': function() { return a; }});
/* harmony import */var _b_js__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./b.js */"./b.js");

 class Test {
    static c = (0, _b_js__WEBPACK_IMPORTED_MODULE__["bb"])();
    static test() {
        _b_js__WEBPACK_IMPORTED_MODULE__["bb"];
    }
}
class Result {
    static test() {
        _b_js__WEBPACK_IMPORTED_MODULE__["cc"];
    }
}
 const a = 3;
},
"./b.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'bb': function() { return bb; }});
 const bb = 2;
 const cc = 3;
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _a_js__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./a.js */"./a.js");

_a_js__WEBPACK_IMPORTED_MODULE__["a"];
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);