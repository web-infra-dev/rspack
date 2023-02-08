(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./bar.js": function (module, exports, __webpack_require__) {
"use strict";
},
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.es(__webpack_require__("./bar.js"), exports);
const a = 'foo';
__webpack_require__.d(exports, {
    "a": ()=>a
});
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _fooJs = __webpack_require__("./foo.js");
console.log(_fooJs.a);
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);