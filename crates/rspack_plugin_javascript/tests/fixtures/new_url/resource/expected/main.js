(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./react.svg": function (module, exports, __webpack_require__) {
"use strict";
module.exports = __webpack_require__.p + "5d7c2bf56394b7b7.svg";},
"./vue.svg": function (module, exports, __webpack_require__) {
"use strict";
module.exports = __webpack_require__.p + "5f5ecd0973bd7725.svg";},
"./index.js": function (module, exports, __webpack_require__) {
const imgSrc = new URL(__webpack_require__("./react.svg"), self.location);
const imgSrc2 = __webpack_require__("./vue.svg");
const img = new Image();
img.src = imgSrc.href;
img.src = imgSrc2;
},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);