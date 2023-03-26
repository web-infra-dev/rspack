(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
it("should allow to run a WebAssembly module importing from multiple modules", function() {
    return __webpack_require__.el("./module.js").then(__webpack_require__.bind(__webpack_require__, "./module.js")).then(__webpack_require__.ir).then(function(mod) {
        expect(mod.result).toBe(42);
    });
});
},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);