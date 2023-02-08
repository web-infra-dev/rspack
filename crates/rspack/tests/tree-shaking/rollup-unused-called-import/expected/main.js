(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./dead.js": function (module, exports, __webpack_require__) {
"use strict";
},
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./dead.js");
function __RSPACK_DEFAULT_EXPORT__() {
    return "foo";
}
__webpack_require__.d(exports, {
    "__RSPACK_DEFAULT_EXPORT__": ()=>__RSPACK_DEFAULT_EXPORT__
});
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _fooJs = __webpack_require__.ir(__webpack_require__("./foo.js"));
assert.equal((0, _fooJs.default)(), "foo");
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);