(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
var Foo = function() {
    console.log("side effect");
    this.isFoo = true;
};
const _default = Foo;
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
const _foo = __webpack_require__.interopRequire(__webpack_require__("./foo.js"));
new _foo.default();
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);