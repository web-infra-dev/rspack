(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./data.txt": function (module, exports, __webpack_require__) {
"use strict";
module.exports = "- Isn't Rspack a gamechanging bundler?\n  - Hella yeah!";},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _datatxt = __webpack_require__.ir(__webpack_require__("./data.txt"));
console.log(_datatxt.default);
},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);