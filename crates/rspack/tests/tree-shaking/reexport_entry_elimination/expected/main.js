(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "b", {
    enumerable: true,
    get: ()=>_bJs.default
});
const _bJs = __webpack_require__.ir(__webpack_require__("./b.js"));
},
"./b.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _cJs = __webpack_require__.ir(__webpack_require__("./c.js"));
__webpack_require__.d(exports, {
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
let __RSPACK_DEFAULT_EXPORT__ = 2000 + _cJs.default;
},
"./c.js": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
let __RSPACK_DEFAULT_EXPORT__ = 10;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _aJs = __webpack_require__("./a.js");
_aJs.b;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);