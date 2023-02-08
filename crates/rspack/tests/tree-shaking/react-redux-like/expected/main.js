(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
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
    Provider: ()=>_libJs.default,
    useSelector: ()=>_selectorJs.default
});
const _libJs = __webpack_require__.ir(__webpack_require__("./lib.js"));
const _selectorJs = __webpack_require__.ir(__webpack_require__("./selector.js"));
__webpack_require__.d(exports, {
    "useSelector": ()=>_selectorJs.default,
    "Provider": ()=>_libJs.default
});
},
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.es(__webpack_require__("./app.js"), exports);
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _fooJs = __webpack_require__("./foo.js");
_fooJs.Provider;
_fooJs.useSelector;
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
function Provider() {}
__webpack_require__.d(exports, {
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
let __RSPACK_DEFAULT_EXPORT__ = Provider;
},
"./selector.js": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "useSelector": ()=>useSelector
});
function useSelector() {
    return "";
}
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);