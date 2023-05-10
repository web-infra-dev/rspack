(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
console.log('b');
},
"./d.js": function (module, exports, __webpack_require__) {
console.log('d');
},
"./index.js": function (module, exports, __webpack_require__) {
__webpack_require__("./a.js");
__webpack_require__("./d.js");
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);