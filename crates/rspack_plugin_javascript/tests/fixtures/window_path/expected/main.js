(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
".\\a.js": function (module, exports, __webpack_require__) {
__webpack_require__(".\\b.js");
console.log('./a');
},
".\\b.js": function (module, exports, __webpack_require__) {
console.log('b');
},
".\\index.js": function (module, exports, __webpack_require__) {
__webpack_require__(".\\a.js");
console.log('fff');
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__(".\\index.js"));

}
]);