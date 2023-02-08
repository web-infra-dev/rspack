(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
var foo = function() {
    return 42;
};
let __RSPACK_DEFAULT_EXPORT__ = foo;
__webpack_require__.d(exports, {
    "bar": ()=>bar,
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
function bar() {
    return contrivedExample(foo);
}
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _fooJs = __webpack_require__.ir(__webpack_require__("./foo.js"));
var answer = (0, _fooJs.default)();
(0, _fooJs.bar)();
console.log(answer);
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);