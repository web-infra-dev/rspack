(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./import-module.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _module__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./module */"./module.js");

expect(_module__WEBPACK_IMPORTED_MODULE__["ok"]).toBe(true);
expect(_module__WEBPACK_IMPORTED_MODULE__["ok2"]).toBe(true);
},
"./index.js": function (module, exports, __webpack_require__) {
it("should not threat globals as pure", ()=>{
    __webpack_require__(/* ./import-module */"./import-module.js");
});
},
"./module.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'ok': function() { return ok; }, 'ok2': function() { return ok2; }});
try {
    var x = NOT_DEFINED;
    var y = x;
    var ok = false;
} catch (e) {
    var yep = true;
    var ok = yep;
}
try {
    const b = a;
    var c = b;
    const a = 42;
    var ok2 = false;
    eval(""); // TODO terser has a bug and incorrectly remove this code, eval opts out
} catch (e) {
    var ok2 = true;
}

},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);