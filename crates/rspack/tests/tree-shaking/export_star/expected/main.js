(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./bar.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "bar", {
    enumerable: true,
    get: ()=>_fooJs
});
const _fooJs = __webpack_require__.ir(__webpack_require__("./foo.js"));
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
    a: ()=>a,
    foo: ()=>foo
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
const _fooJs = __webpack_require__("./foo.js");
_fooJs.bar.a;
(0, _fooJs.c)();
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
__webpack_require__.es(__webpack_require__("./foo.js"), exports);
__webpack_require__.es(__webpack_require__("./bar.js"), exports);
const c = 103330;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);