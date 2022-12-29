(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _inner = __webpack_require__("./inner.js");
const _module = __webpack_require__("./module.js");
it("export should be unused when only unused functions use it", ()=>{
    expect((0, _module.y)("a")).toBe("okBAA");
    expect(_inner.exportAUsed).toBe(true);
    expect(_inner.exportBUsed).toBe(true);
    if (process.env.NODE_ENV === "production") expect(_inner.exportCUsed).toBe(false);
    return __webpack_require__.e("chunk_js").then(__webpack_require__.bind(__webpack_require__, "./chunk.js")).then(__webpack_require__.interopRequire);
});
},
"./inner.js": function (module, exports, __webpack_require__) {
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
    A: ()=>A,
    B: ()=>B,
    exportAUsed: ()=>exportAUsed,
    exportBUsed: ()=>exportBUsed,
    exportCUsed: ()=>exportCUsed
});
function A(s) {
    return s + "A";
}
function B(s) {
    return s + "B";
}
const exportAUsed = __webpack_exports_info__.A.used;
const exportBUsed = __webpack_exports_info__.B.used;
const exportCUsed = __webpack_exports_info__.C.used;
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
    x: ()=>x,
    y: ()=>y
});
const _inner = __webpack_require__("./inner.js");
function x(type) {
    switch(type){
        case "a":
            return withA("b");
        case "b":
            return withB("c");
        case "c":
            return "ok";
    }
}
function y(v) {
    return withA(v);
}
function withA(v) {
    const value = x(v);
    return (0, _inner.A)(value);
}
function withB(v) {
    const value = x(v);
    return (0, _inner.B)(value);
}
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);