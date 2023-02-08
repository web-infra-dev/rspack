(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _depJsA = __webpack_require__("./dep.js?a");
__webpack_require__.d(exports, {
    "abc": ()=>abc
});
function abc() {
    return _depJsA.x;
}
},
"./b.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./dep.js?b");
},
"./c.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _depJsC = __webpack_require__("./dep.js?c");
function abc() {
    return _depJsC.x;
}
abc();
},
"./d.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _depJsD = __webpack_require__("./dep.js?d");
__webpack_require__.d(exports, {
    "def": ()=>def
});
class def {
    method() {
        return _depJsD.x;
    }
}
},
"./dep.js?a": function (module, exports, __webpack_require__) {
"use strict";
const x = "x";
__webpack_require__.d(exports, {
    "x": ()=>x,
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
let __RSPACK_DEFAULT_EXPORT__ = __webpack_exports_info__.x.used;
},
"./dep.js?b": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
let __RSPACK_DEFAULT_EXPORT__ = __webpack_exports_info__.x.used;
},
"./dep.js?c": function (module, exports, __webpack_require__) {
"use strict";
const x = "x";
__webpack_require__.d(exports, {
    "x": ()=>x,
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
let __RSPACK_DEFAULT_EXPORT__ = __webpack_exports_info__.x.used;
},
"./dep.js?d": function (module, exports, __webpack_require__) {
"use strict";
const x = "x";
__webpack_require__.d(exports, {
    "default": ()=>__RSPACK_DEFAULT_EXPORT__,
    "x": ()=>x
});
let __RSPACK_DEFAULT_EXPORT__ = __webpack_exports_info__.x.used;
},
"./dep.js?e": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
let __RSPACK_DEFAULT_EXPORT__ = __webpack_exports_info__.x.used;
},
"./dep.js?f": function (module, exports, __webpack_require__) {
"use strict";
const x = "x";
__webpack_require__.d(exports, {
    "x": ()=>x,
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
let __RSPACK_DEFAULT_EXPORT__ = __webpack_exports_info__.x.used;
},
"./e.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./dep.js?e");
},
"./f.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _depJsF = __webpack_require__("./dep.js?f");
class def {
    method() {
        return _depJsF.x;
    }
}
new def().method();
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _aJs = __webpack_require__.ir(__webpack_require__("./a.js"));
__webpack_require__("./b.js");
__webpack_require__("./c.js");
const _dJs = __webpack_require__.ir(__webpack_require__("./d.js"));
__webpack_require__("./e.js");
__webpack_require__("./f.js");
const _depJsA = __webpack_require__.ir(__webpack_require__("./dep.js?a"));
const _depJsB = __webpack_require__.ir(__webpack_require__("./dep.js?b"));
const _depJsC = __webpack_require__.ir(__webpack_require__("./dep.js?c"));
const _depJsD = __webpack_require__.ir(__webpack_require__("./dep.js?d"));
const _depJsE = __webpack_require__.ir(__webpack_require__("./dep.js?e"));
const _depJsF = __webpack_require__.ir(__webpack_require__("./dep.js?f"));
it("should generate valid code", ()=>{
    expect((0, _aJs.default)()).toBe("x");
    expect(new _dJs.default().method()).toBe("x");
});
it("a should be used", ()=>{
    expect(_depJsA.default).toBe(true);
});
it("b should be unused", ()=>{
    expect(_depJsB.default).toBe(false);
});
it("c should be used", ()=>{
    expect(_depJsC.default).toBe(true);
});
it("d should be used", ()=>{
    expect(_depJsD.default).toBe(true);
});
it("e should be unused", ()=>{
    expect(_depJsE.default).toBe(false);
});
it("f should be used", ()=>{
    expect(_depJsF.default).toBe(true);
});
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);