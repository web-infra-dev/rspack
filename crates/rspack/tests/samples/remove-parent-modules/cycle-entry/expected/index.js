(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["index"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./shared.js");
Promise.all([
    __webpack_require__.e("index"),
    __webpack_require__.e("runtime")
]).then(__webpack_require__.bind(__webpack_require__, "./index.js")).then(__webpack_require__.ir);
console.log('index1');
},
"./shared.js": function (module, exports, __webpack_require__) {
console.log('shared');
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);