(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./b.js?x": function (module, exports, __webpack_require__) {
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
const _bJsx = __webpack_require__("./b.jsx");
const _bJsX = __webpack_require__("./b.js?x");
_bJsX.a;
_bJsx.a;
console.log("hello, world");
},
"./b.jsx": function (module, exports, __webpack_require__) {
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

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);