(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _inner = __webpack_require__("./inner.js");
var _module = __webpack_require__("./module.js");
it("export should be unused when only unused functions use it", ()=>{
    expect((0, _module.y)("a")).toBe("okBAA");
    expect(_inner.exportAUsed).toBe(true);
    expect(_inner.exportBUsed).toBe(true);
    expect(_inner.exportCUsed).toBe(false);
    return __webpack_require__.el("./chunk.js").then(__webpack_require__.bind(__webpack_require__, "./chunk.js")).then(__webpack_require__.ir);
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
    A: function() {
        return A;
    },
    B: function() {
        return B;
    },
    exportAUsed: function() {
        return exportAUsed;
    },
    exportBUsed: function() {
        return exportBUsed;
    },
    exportCUsed: function() {
        return exportCUsed;
    }
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
    x: function() {
        return x;
    },
    y: function() {
        return y;
    }
});
var _inner = __webpack_require__("./inner.js");
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
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);