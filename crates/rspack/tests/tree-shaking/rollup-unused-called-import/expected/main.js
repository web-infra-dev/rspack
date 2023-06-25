(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./dead.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
function __WEBPACK_DEFAULT_EXPORT__() {
    return "dead";
}
},
"./foo.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }});
/* harmony import */var _dead__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./dead */"./dead.js");

function __WEBPACK_DEFAULT_EXPORT__() {
    return "foo";
}
 function foodead() {
    return "foo" + (0, _dead__WEBPACK_IMPORTED_MODULE__["default"])();
}
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _foo__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./foo */"./foo.js");

assert.equal((0, _foo__WEBPACK_IMPORTED_MODULE__["default"])(), "foo");
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);