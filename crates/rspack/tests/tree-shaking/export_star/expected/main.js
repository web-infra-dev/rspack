(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./bar.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "bar", {
    enumerable: true,
    get: function() {
        return _foo;
    }
});
var _foo = __webpack_require__.ir(__webpack_require__("./foo.js"));
__webpack_require__.es(__webpack_require__("./result.js"), exports);
},
"./foo.js": function (module, exports, __webpack_require__) {
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
    a: function() {
        return a;
    },
    foo: function() {
        return foo;
    }
});
__webpack_require__.es(__webpack_require__("./bar.js"), exports);
__webpack_require__.es(__webpack_require__("./result.js"), exports);
const a = 3;
const foo = 3;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _foo = __webpack_require__("./foo.js");
_foo.bar.a;
(0, _foo.c)();
},
"./result.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "c", {
    enumerable: true,
    get: function() {
        return c;
    }
});
__webpack_require__.es(__webpack_require__("./foo.js"), exports);
__webpack_require__.es(__webpack_require__("./bar.js"), exports);
const c = 103330;
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);