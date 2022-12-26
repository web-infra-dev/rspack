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
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    default: ()=>_default,
    test: ()=>test
});
const _package = __webpack_require__("./package/index.js");
const _default = _package.a;
function test() {}
},
"./package/index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: ()=>a
});
function a() {
    return 42;
}
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);