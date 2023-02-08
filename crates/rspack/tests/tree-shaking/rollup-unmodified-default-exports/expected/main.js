(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
var Foo = function() {
    console.log("side effect");
    this.isFoo = true;
};
__webpack_require__.d(exports, {
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
let __RSPACK_DEFAULT_EXPORT__ = Foo;
Foo.prototype = {
    answer: function() {
        return 42;
    }
};
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _fooJs = __webpack_require__.ir(__webpack_require__("./foo.js"));
new _fooJs.default();
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);