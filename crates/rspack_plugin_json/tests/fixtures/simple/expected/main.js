(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _json_json__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./json.json */"./json.json");

console.log(_json_json__WEBPACK_IMPORTED_MODULE_0_);
},
"./json.json": function (module, exports, __webpack_require__) {
module.exports = {
  "hello": "world"
}
;},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);