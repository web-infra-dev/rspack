(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
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
"./c.js": function (module, exports, __webpack_require__) {
"use strict";
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _aJs = __webpack_require__("./a.js");
__webpack_require__("./c.js");
_aJs.a;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);