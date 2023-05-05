(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _script = __webpack_require__("./package1/script.js");
var _script2 = __webpack_require__("./package1/script2.js");
var _script1 = __webpack_require__("./package2/script.js");
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
    get: function() {
        return mod;
    }
});
__webpack_require__("./package1/script.js");
var _script = __webpack_require__.ir(__webpack_require__("./package2/script.js"));
const mod = _script.default;
},
"./package1/script.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "exportDefaultUsed", {
    enumerable: true,
    get: function() {
        return exportDefaultUsed;
    }
});
__webpack_require__.es(__webpack_require__("./package1/script1.js"), exports);
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
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "exportDefaultUsed", {
    enumerable: true,
    get: function() {
        return exportDefaultUsed;
    }
});
const exportDefaultUsed = __webpack_exports_info__.default.used;
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
    default: function() {
        return _default;
    },
    exportDefaultUsed: function() {
        return exportDefaultUsed;
    }
});
var _script1 = __webpack_require__.ir(__webpack_require__.es(__webpack_require__("./package2/script1.js"), exports));
var _default = _script1.default;
const exportDefaultUsed = __webpack_exports_info__.default.used;
},
"./package2/script1.js": function (module, exports, __webpack_require__) {
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
var _default = 1;
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);