(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _dep_a__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./dep?a */"./dep.js?a");
/* harmony import */var _d__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./d */"./d.js");
/* harmony import */var _e__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./e */"./e.js");
/* harmony import */var _f__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./f */"./f.js");
/* harmony import */var _a__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./a */"./a.js");
/* harmony import */var _b__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./b */"./b.js");
/* harmony import */var _c__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./c */"./c.js");
/* harmony import */var _dep_b__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./dep?b */"./dep.js?b");
/* harmony import */var _dep_c__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./dep?c */"./dep.js?c");
/* harmony import */var _dep_d__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./dep?d */"./dep.js?d");
/* harmony import */var _dep_e__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./dep?e */"./dep.js?e");
/* harmony import */var _dep_f__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./dep?f */"./dep.js?f");












it("should generate valid code", ()=>{
    expect(_a__WEBPACK_IMPORTED_MODULE__["default"]()).toBe("x");
    expect(new _d__WEBPACK_IMPORTED_MODULE__["default"]().method()).toBe("x");
});
it("a should be used", ()=>{
    expect(_dep_a__WEBPACK_IMPORTED_MODULE__["default"]).toBe(true);
});
it("b should be unused", ()=>{
    expect(_dep_b__WEBPACK_IMPORTED_MODULE__["default"]).toBe(false);
});
it("c should be used", ()=>{
    expect(_dep_c__WEBPACK_IMPORTED_MODULE__["default"]).toBe(true);
});
it("d should be used", ()=>{
    expect(_dep_d__WEBPACK_IMPORTED_MODULE__["default"]).toBe(true);
});
it("e should be unused", ()=>{
    expect(_dep_e__WEBPACK_IMPORTED_MODULE__["default"]).toBe(false);
});
it("f should be used", ()=>{
    expect(_dep_f__WEBPACK_IMPORTED_MODULE__["default"]).toBe(true);
});
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);