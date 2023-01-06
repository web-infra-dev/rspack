(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
console.log('hello, world');
__webpack_require__.e("a_js").then(__webpack_require__.bind(__webpack_require__, "./a.js")).then(__webpack_require__.ir);
__webpack_require__.e("b_js").then(__webpack_require__.bind(__webpack_require__, "./b.js")).then(__webpack_require__.ir);
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);