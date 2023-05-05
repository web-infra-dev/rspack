(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _jsonjson = __webpack_require__.ir(__webpack_require__("./json.json"));
console.log(_jsonjson.default);
},
"./json.json": function (module, exports, __webpack_require__) {
module.exports = {
  "hello": "world"
}
;},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);