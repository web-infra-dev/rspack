(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>abc
});
const _depA = __webpack_require__("./dep.jsa");
function abc() {
    return _depA.x;
}
},
"./b.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _depB = __webpack_require__("./dep.jsb");
},
"./c.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _depC = __webpack_require__("./dep.jsc");
function abc() {
    return _depC.x;
}
abc();
},
"./d.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>def
});
const _depD = __webpack_require__("./dep.jsd");
class def {
    method() {
        return _depD.x;
    }
}
},
"./dep.jsa": function (module, exports, __webpack_require__) {
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
    default: ()=>_default
});
const x = "x";
const _default = __webpack_exports_info__.x.used;
},
"./dep.jsb": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
const _default = __webpack_exports_info__.x.used;
},
"./dep.jsc": function (module, exports, __webpack_require__) {
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
    default: ()=>_default
});
const x = "x";
const _default = __webpack_exports_info__.x.used;
},
"./dep.jsd": function (module, exports, __webpack_require__) {
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
    default: ()=>_default
});
const x = "x";
const _default = __webpack_exports_info__.x.used;
},
"./dep.jse": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
const _default = __webpack_exports_info__.x.used;
},
"./dep.jsf": function (module, exports, __webpack_require__) {
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
    default: ()=>_default
});
const x = "x";
const _default = __webpack_exports_info__.x.used;
},
"./e.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _depE = __webpack_require__("./dep.jse");
},
"./f.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _depF = __webpack_require__("./dep.jsf");
class def {
    method() {
        return _depF.x;
    }
}
new def().method();
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _a = __webpack_require__.interopRequire(__webpack_require__("./a.js"));
__webpack_require__("./b.js");
__webpack_require__("./c.js");
const _d = __webpack_require__.interopRequire(__webpack_require__("./d.js"));
__webpack_require__("./e.js");
__webpack_require__("./f.js");
const _depA = __webpack_require__.interopRequire(__webpack_require__("./dep.jsa"));
const _depB = __webpack_require__.interopRequire(__webpack_require__("./dep.jsb"));
const _depC = __webpack_require__.interopRequire(__webpack_require__("./dep.jsc"));
const _depD = __webpack_require__.interopRequire(__webpack_require__("./dep.jsd"));
const _depE = __webpack_require__.interopRequire(__webpack_require__("./dep.jse"));
const _depF = __webpack_require__.interopRequire(__webpack_require__("./dep.jsf"));
it("should generate valid code", ()=>{
    expect((0, _a.default)()).toBe("x");
    expect(new _d.default().method()).toBe("x");
});
it("a should be used", ()=>{
    expect(_depA.default).toBe(true);
});
if (process.env.NODE_ENV === "production") it("b should be unused", ()=>{
    expect(_depB.default).toBe(false);
});
it("c should be used", ()=>{
    expect(_depC.default).toBe(true);
});
if (process.env.NODE_ENV === "production") {
    it("d should be used", ()=>{
        expect(_depD.default).toBe(true);
    });
    it("e should be unused", ()=>{
        expect(_depE.default).toBe(false);
    });
}
it("f should be used", ()=>{
    expect(_depF.default).toBe(true);
});
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);