(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./b.jsx?8a8c": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: ()=>a
});
const a = 3;
},
"./b.jsx?973e": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: ()=>a
});
const a = 3;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _bJsx973E = __webpack_require__("./b.jsx?973e");
const _bJsx8A8C = __webpack_require__("./b.jsx?8a8c");
_bJsx8A8C.a;
_bJsx973E.a;
console.log("hello, world");
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);