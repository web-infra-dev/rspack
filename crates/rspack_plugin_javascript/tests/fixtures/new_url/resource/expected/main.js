(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
const imgSrc = new URL(/* asset import */__webpack_require__(/* ./react.svg */"./react.svg"), __webpack_require__.b);
const imgSrc2 = __webpack_require__(/* ./vue.svg */"./vue.svg");
const img = new Image();
img.src = imgSrc.href;
img.src = imgSrc2;
},
"./react.svg": function (module, exports, __webpack_require__) {
module.exports = __webpack_require__.p + "751e775393b8ce2a.svg";},
"./vue.svg": function (module, exports, __webpack_require__) {
module.exports = __webpack_require__.p + "ea9452531f910819.svg";},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);