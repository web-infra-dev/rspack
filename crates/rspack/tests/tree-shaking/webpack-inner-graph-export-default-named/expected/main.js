(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return abc;
    }
});
var _depJsA = __webpack_require__("./dep.js?a");
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
var _depJsC = __webpack_require__("./dep.js?c");
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
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return def;
    }
});
var _depJsD = __webpack_require__("./dep.js?d");
class def {
    method() {
        return _depJsD.x;
    }
}
},
"./dep.js?a": function (module, exports, __webpack_require__) {
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
    default: function() {
        return _default;
    }
});
const x = "x";
var _default = __webpack_exports_info__.x.used;
},
"./dep.js?b": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _default = __webpack_exports_info__.x.used;
},
"./dep.js?c": function (module, exports, __webpack_require__) {
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
    default: function() {
        return _default;
    }
});
const x = "x";
var _default = __webpack_exports_info__.x.used;
},
"./dep.js?d": function (module, exports, __webpack_require__) {
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
    default: function() {
        return _default;
    }
});
const x = "x";
var _default = __webpack_exports_info__.x.used;
},
"./dep.js?e": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _default = __webpack_exports_info__.x.used;
},
"./dep.js?f": function (module, exports, __webpack_require__) {
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
    default: function() {
        return _default;
    }
});
const x = "x";
var _default = __webpack_exports_info__.x.used;
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
var _depJsF = __webpack_require__("./dep.js?f");
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
var _aJs = __webpack_require__.ir(__webpack_require__("./a.js"));
__webpack_require__("./b.js");
__webpack_require__("./c.js");
var _dJs = __webpack_require__.ir(__webpack_require__("./d.js"));
__webpack_require__("./e.js");
__webpack_require__("./f.js");
var _depJsA = __webpack_require__.ir(__webpack_require__("./dep.js?a"));
var _depJsB = __webpack_require__.ir(__webpack_require__("./dep.js?b"));
var _depJsC = __webpack_require__.ir(__webpack_require__("./dep.js?c"));
var _depJsD = __webpack_require__.ir(__webpack_require__("./dep.js?d"));
var _depJsE = __webpack_require__.ir(__webpack_require__("./dep.js?e"));
var _depJsF = __webpack_require__.ir(__webpack_require__("./dep.js?f"));
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
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);