(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
it("should be able to load package without side effects where modules are unused", ()=>{
    __webpack_require__("./module.js");
});
},
"./module.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _indexJs = __webpack_require__("./package/index.js");
let __RSPACK_DEFAULT_EXPORT__ = _indexJs.a;
__webpack_require__.d(exports, {
    "test": ()=>test,
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
function test() {}
},
"./package/index.js": function (module, exports, __webpack_require__) {
"use strict";
function a() {
    return 42;
}
__webpack_require__.d(exports, {
    "a": ()=>a
});
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);