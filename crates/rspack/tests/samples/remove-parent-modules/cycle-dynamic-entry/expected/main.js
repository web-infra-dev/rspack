(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
__webpack_require__.el("./dynamic-1.js").then(__webpack_require__.bind(__webpack_require__, "./dynamic-1.js")).then(__webpack_require__.ir);
__webpack_require__.el("./dynamic-2.js").then(__webpack_require__.bind(__webpack_require__, "./dynamic-2.js")).then(__webpack_require__.ir);
console.log('index');
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);