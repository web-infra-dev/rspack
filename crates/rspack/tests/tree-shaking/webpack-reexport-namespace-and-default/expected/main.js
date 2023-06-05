(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _package1_script2__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./package1/script2 */"./package1/script2.js");
/* harmony import */var _package2_script__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./package2/script */"./package2/script.js");
/* harmony import */var _package1_script__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./package1/script */"./package1/script.js");



it("should load module correctly", ()=>{
    __webpack_require__(/* ./module */"./module.js");
});
// if (process.env.NODE_ENV === "production") {
it("default export should be unused", ()=>{
    expect(_package1_script__WEBPACK_IMPORTED_MODULE__["exportDefaultUsed"]).toBe(false);
    expect(_package1_script2__WEBPACK_IMPORTED_MODULE__["exportDefaultUsed"]).toBe(false);
});
// }
it("default export should be used", ()=>{
    expect(_package2_script__WEBPACK_IMPORTED_MODULE__["exportDefaultUsed"]).toBe(true);
});
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);