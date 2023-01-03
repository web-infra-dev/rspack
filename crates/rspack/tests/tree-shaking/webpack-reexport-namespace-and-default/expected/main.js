(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _script = __webpack_require__("./package1/script.js");
const _script2 = __webpack_require__("./package1/script2.js");
const _script1 = __webpack_require__("./package2/script.js");
it("should load module correctly", ()=>{
    __webpack_require__("./module.js");
});
it("default export should be unused", ()=>{
    expect(_script.exportDefaultUsed).toBe(false);
    expect(_script2.exportDefaultUsed).toBe(false);
});
it("default export should be used", ()=>{
    expect(_script1.exportDefaultUsed).toBe(true);
});
},
"./module.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "mod", {
    enumerable: true,
    get: ()=>mod
});
const _script = __webpack_require__.interopRequire(__webpack_require__("./package1/script.js"));
const _script1 = __webpack_require__.interopRequire(__webpack_require__("./package2/script.js"));
const mod = _script1.default;
},
"./package1/script.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "exportDefaultUsed", {
    enumerable: true,
    get: ()=>exportDefaultUsed
});
const _script1 = __webpack_require__.interopRequire(__webpack_require__.exportStar(__webpack_require__("./package1/script1.js"), exports));
const exportDefaultUsed = __webpack_exports_info__.default.used;
},
"./package1/script1.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
__webpack_require__.exportStar(__webpack_require__("./package1/script2.js"), exports);
const _default = 1;
},
"./package1/script2.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "exportDefaultUsed", {
    enumerable: true,
    get: ()=>exportDefaultUsed
});
const _script3 = __webpack_require__.interopRequire(__webpack_require__.exportStar(__webpack_require__("./package1/script3.js"), exports));
const exportDefaultUsed = __webpack_exports_info__.default.used;
},
"./package1/script3.js": function (module, exports, __webpack_require__) {
"use strict";
},
"./package2/script.js": function (module, exports, __webpack_require__) {
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
    exportDefaultUsed: ()=>exportDefaultUsed
});
const _script1 = __webpack_require__.interopRequire(__webpack_require__.exportStar(__webpack_require__("./package2/script1.js"), exports));
const _default = _script1.default;
const exportDefaultUsed = __webpack_exports_info__.default.used;
},
"./package2/script1.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
const _default = 1;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);