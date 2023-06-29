(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
__webpack_require__.el(/* ./dynamic-1 */"./dynamic-1.js").then(__webpack_require__.bind(__webpack_require__, /* ./dynamic-1 */"./dynamic-1.js"));
__webpack_require__.el(/* ./dynamic-2 */"./dynamic-2.js").then(__webpack_require__.bind(__webpack_require__, /* ./dynamic-2 */"./dynamic-2.js"));
console.log('index');
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);