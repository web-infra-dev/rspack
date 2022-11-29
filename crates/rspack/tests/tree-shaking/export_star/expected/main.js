(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./bar.js": function (module, exports, __webpack_require__) {
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
    b: ()=>b,
    bar: ()=>_foo
});
const _foo = __webpack_require__.interopRequire(__webpack_require__("./foo.js"));
__webpack_require__.exportStar(__webpack_require__("./result.js"), exports);
function b() {}
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
    a: ()=>a,
    foo: ()=>foo
});
__webpack_require__.exportStar(__webpack_require__("./bar.js"), exports);
__webpack_require__.exportStar(__webpack_require__("./result.js"), exports);
const a = 3;
const foo = 3;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _foo = __webpack_require__("./foo.js");
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
    get: ()=>c
});
__webpack_require__.exportStar(__webpack_require__("./foo.js"), exports);
__webpack_require__.exportStar(__webpack_require__("./bar.js"), exports);
const c = 103330;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);