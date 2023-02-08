(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./import-module.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _moduleJs = __webpack_require__("./module.js");
expect(_moduleJs.ok).toBe(true);
expect(_moduleJs.ok2).toBe(true);
},
"./index.js": function (module, exports, __webpack_require__) {
it("should not threat globals as pure", ()=>{
    __webpack_require__("./import-module.js");
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
    ok: ()=>ok,
    ok2: ()=>ok2
});
try {
    NOT_DEFINED;
    var ok = false;
} catch (e) {
    var yep = true;
    var ok = yep;
}
try {
    var ok2 = false;
    eval("");
} catch (e) {
    var ok2 = true;
}
__webpack_require__.d(exports, {
    "ok2": ()=>ok2,
    "ok": ()=>ok
});
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);