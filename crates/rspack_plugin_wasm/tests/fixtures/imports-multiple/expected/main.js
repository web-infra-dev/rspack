(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
(async function() {
    return __webpack_require__.el(/* ./module */"./module.js").then(__webpack_require__.bind(__webpack_require__, /* ./module */"./module.js")).then(function(mod) {
        if (mod.result !== 42) throw new Error('panic');
    });
})();
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);