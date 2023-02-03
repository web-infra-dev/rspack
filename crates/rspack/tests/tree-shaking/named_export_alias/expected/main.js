(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./export.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>a
});
var a = function test() {
    something;
};
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _exportJs = __webpack_require__.ir(__webpack_require__("./export.js"));
_exportJs.default;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);