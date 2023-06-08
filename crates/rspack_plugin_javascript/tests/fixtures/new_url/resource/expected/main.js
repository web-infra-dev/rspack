(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
const imgSrc = new URL(__webpack_require__("./react.svg"), __webpack_require__.b);
const imgSrc2 = __webpack_require__("./vue.svg");
const img = new Image();
img.src = imgSrc.href;
img.src = imgSrc2;
},
"./react.svg": function (module, exports, __webpack_require__) {
module.exports = __webpack_require__.p + "5b6d4936f12d1301.svg";},
"./vue.svg": function (module, exports, __webpack_require__) {
module.exports = __webpack_require__.p + "8c7236080ec784a5.svg";},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);