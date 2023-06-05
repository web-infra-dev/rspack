(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _module__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./module */"./module.js");

it("should be able to handle circular referenced", ()=>{
    expect(_module__WEBPACK_IMPORTED_MODULE__["x"]()).toEqual([
        _module__WEBPACK_IMPORTED_MODULE__["y"],
        _module__WEBPACK_IMPORTED_MODULE__["z"]
    ]);
    const [_a, b, c, d] = _module__WEBPACK_IMPORTED_MODULE__["a"]();
    expect(b()).toEqual([
        _module__WEBPACK_IMPORTED_MODULE__["a"],
        b,
        c,
        d
    ]);
    expect(c()).toEqual([
        _module__WEBPACK_IMPORTED_MODULE__["a"],
        b,
        c,
        d
    ]);
    expect(d()).toEqual([
        _module__WEBPACK_IMPORTED_MODULE__["a"],
        b,
        c,
        d
    ]);
    const [f2, f4] = _module__WEBPACK_IMPORTED_MODULE__["f3"]();
    const [f1, _f3] = f2();
    expect(_f3).toBe(_module__WEBPACK_IMPORTED_MODULE__["f3"]);
    expect(_module__WEBPACK_IMPORTED_MODULE__["f3"]()).toEqual(f1());
    expect(f2()).toEqual(f4());
});
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);