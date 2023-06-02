(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["index"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./shared.js");
__webpack_require__.el("./index.js").then(__webpack_require__.bind(__webpack_require__, "./index.js")).then(__webpack_require__.ir);
console.log('index1');
},
"./shared.js": function (module, exports, __webpack_require__) {
console.log('shared');
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);