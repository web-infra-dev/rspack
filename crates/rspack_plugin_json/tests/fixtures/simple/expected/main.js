(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _jsonJson = __webpack_require__.ir(__webpack_require__("./json.json"));
console.log(_jsonJson.default);
},
"./json.json": function (module, exports, __webpack_require__) {
"use strict";
module.exports = {
  "hello": "world"
}
;},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);