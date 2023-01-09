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
const _depJsa = __webpack_require__("./dep.jsa");
function abc() {
    return _depJsa.x;
}
},
"./b.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _depJsb = __webpack_require__("./dep.jsb");
},
"./c.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _depJsc = __webpack_require__("./dep.jsc");
function abc() {
    return _depJsc.x;
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
const _depJsd = __webpack_require__("./dep.jsd");
class def {
    method() {
        return _depJsd.x;
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
const _depJse = __webpack_require__("./dep.jse");
},
"./f.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _depJsf = __webpack_require__("./dep.jsf");
class def {
    method() {
        return _depJsf.x;
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
const _depJsa = __webpack_require__.ir(__webpack_require__("./dep.jsa"));
const _depJsb = __webpack_require__.ir(__webpack_require__("./dep.jsb"));
const _depJsc = __webpack_require__.ir(__webpack_require__("./dep.jsc"));
const _depJsd = __webpack_require__.ir(__webpack_require__("./dep.jsd"));
const _depJse = __webpack_require__.ir(__webpack_require__("./dep.jse"));
const _depJsf = __webpack_require__.ir(__webpack_require__("./dep.jsf"));
it("should generate valid code", ()=>{
    expect((0, _aJs.default)()).toBe("x");
    expect(new _dJs.default().method()).toBe("x");
});
it("a should be used", ()=>{
    expect(_depJsa.default).toBe(true);
});
if (process.env.NODE_ENV === "production") it("b should be unused", ()=>{
    expect(_depJsb.default).toBe(false);
});
it("c should be used", ()=>{
    expect(_depJsc.default).toBe(true);
});
if (process.env.NODE_ENV === "production") {
    it("d should be used", ()=>{
        expect(_depJsd.default).toBe(true);
    });
    it("e should be unused", ()=>{
        expect(_depJse.default).toBe(false);
    });
}
it("f should be used", ()=>{
    expect(_depJsf.default).toBe(true);
});
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);