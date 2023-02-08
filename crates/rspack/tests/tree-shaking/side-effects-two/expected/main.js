(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "something", {
    enumerable: true,
    get: ()=>_libJs.default
});
const _libJs = __webpack_require__.ir(__webpack_require__("./lib.js"));
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _appJs = __webpack_require__("./app.js");
(0, _appJs.something)();
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "__RSPACK_DEFAULT_EXPORT__": ()=>__RSPACK_DEFAULT_EXPORT__
});
function __RSPACK_DEFAULT_EXPORT__() {}
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);