(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "foo", {
    enumerable: true,
    get: ()=>foo
});
var foo = "lol";
__webpack_require__.d(exports, {
    "foo": ()=>foo
});
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _fooJs = __webpack_require__("./foo.js");
console.log(_fooJs.foo);
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);