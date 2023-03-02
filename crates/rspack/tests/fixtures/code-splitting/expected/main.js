(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
console.log('hello, world');
__webpack_require__.el("./a.js").then(__webpack_require__.bind(__webpack_require__, "./a.js")).then(__webpack_require__.ir);
__webpack_require__.el("./b.js").then(__webpack_require__.bind(__webpack_require__, "./b.js")).then(__webpack_require__.ir);
},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);