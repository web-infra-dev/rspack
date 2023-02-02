(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["chunk_js"], {
"./chunk.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./inner.js");
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

}]);