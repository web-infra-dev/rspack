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
    default: function() {
        return _default;
    },
    test: function() {
        return test;
    }
});
var _indexJs = __webpack_require__("./package/index.js");
var _default = _indexJs.a;
function test() {}
},
"./package/index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: function() {
        return a;
    }
});
function a() {
    return 42;
}
},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);