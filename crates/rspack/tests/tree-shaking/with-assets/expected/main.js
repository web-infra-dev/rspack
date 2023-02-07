(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: ()=>a
});
const a = 3;
},
"./a.svg": function (module, exports, __webpack_require__) {
"use strict";
module.exports = "data:image/svg+xml;base64,";},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _aJs = __webpack_require__("./a.js");
const _aSvg = __webpack_require__.ir(__webpack_require__("./a.svg"));
_aJs.a;
_aSvg.default;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);