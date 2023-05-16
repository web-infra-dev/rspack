(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["index"], {
"./i-1.js": function (module, exports, __webpack_require__) {
console.log('i-1');
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./shared.js");
__webpack_require__("./i-1.js");
__webpack_require__.el("./a.js").then(__webpack_require__.bind(__webpack_require__, "./a.js")).then(__webpack_require__.ir);
console.log('index');
},
"./shared.js": function (module, exports, __webpack_require__) {
console.log('shared');
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);