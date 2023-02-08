(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _scriptJs = __webpack_require__("./package1/script.js");
const _script2Js = __webpack_require__("./package1/script2.js");
const _scriptJs1 = __webpack_require__("./package2/script.js");
it("should load module correctly", ()=>{
    __webpack_require__("./module.js");
});
it("default export should be unused", ()=>{
    expect(_scriptJs.exportDefaultUsed).toBe(false);
    expect(_script2Js.exportDefaultUsed).toBe(false);
});
it("default export should be used", ()=>{
    expect(_scriptJs1.exportDefaultUsed).toBe(true);
});
},
"./module.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _scriptJs = __webpack_require__.ir(__webpack_require__("./package1/script.js"));
const _scriptJs1 = __webpack_require__.ir(__webpack_require__("./package2/script.js"));
__webpack_require__.d(exports, {
    "mod": ()=>mod
});
const mod = _scriptJs1.default;
},
"./package1/script.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.es(__webpack_require__("./package1/script1.js"), exports);
__webpack_require__.d(exports, {
    "exportDefaultUsed": ()=>exportDefaultUsed
});
const exportDefaultUsed = __webpack_exports_info__.default.used;
},
"./package1/script1.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.es(__webpack_require__("./package1/script2.js"), exports);
},
"./package1/script2.js": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "exportDefaultUsed": ()=>exportDefaultUsed
});
const exportDefaultUsed = __webpack_exports_info__.default.used;
},
"./package2/script.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _script1Js = __webpack_require__.ir(__webpack_require__.es(__webpack_require__("./package2/script1.js"), exports));
let __RSPACK_DEFAULT_EXPORT__ = _script1Js.default;
__webpack_require__.d(exports, {
    "exportDefaultUsed": ()=>exportDefaultUsed,
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
const exportDefaultUsed = __webpack_exports_info__.default.used;
},
"./package2/script1.js": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
let __RSPACK_DEFAULT_EXPORT__ = 1;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);