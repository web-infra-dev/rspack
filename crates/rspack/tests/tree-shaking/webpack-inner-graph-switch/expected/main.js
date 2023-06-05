(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
it("should generate correct code when pure expressions are in dead branches", ()=>{
    __webpack_require__(/* ./import-module */"./import-module.js").test();
    return Promise.all([
        __webpack_require__.el(/* ./some-module */"./some-module.js").then(__webpack_require__.bind(__webpack_require__, /* ./some-module */"./some-module.js")).then(__webpack_require__.ir),
        __webpack_require__.el(/* ./chunk */"./chunk.js").then(__webpack_require__.bind(__webpack_require__, /* ./chunk */"./chunk.js")).then(__webpack_require__.ir)
    ]);
});
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);