(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
try {
    __webpack_require__("?2089");
} catch (e) {}
},
"?2089": function (module, exports, __webpack_require__) {
throw new Error("Failed to resolve ./abc.js in /home/wenxin/codes/rspack/crates/rspack_plugin_javascript/tests/fixtures/try_require");
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);