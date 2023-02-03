(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "c", {
    enumerable: true,
    get: ()=>c
});
const c = 'a';
},
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
    a: ()=>a,
    c: ()=>c
});
const a = 'bar';
const c = 'bar';
},
"./c.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: ()=>a
});
__webpack_require__.es(__webpack_require__("./foo.js"), exports);
__webpack_require__.es(__webpack_require__("./bar.js"), exports);
const a = 3;
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
    b: ()=>b
});
__webpack_require__.es(__webpack_require__("./a.js"), exports);
const a = 'foo';
const b = 'foo';
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _cJs = __webpack_require__("./c.js");
_cJs.a;
_cJs.b;
_cJs.c;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);