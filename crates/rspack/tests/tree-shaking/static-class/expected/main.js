(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _bJs = __webpack_require__("./b.js");
class Test {
    static c = (0, _bJs.bb)();
    static test() {
        _bJs.bb;
    }
}
__webpack_require__.d(exports, {
    "a": ()=>a
});
const a = 3;
},
"./b.js": function (module, exports, __webpack_require__) {
"use strict";
const bb = 2;
__webpack_require__.d(exports, {
    "bb": ()=>bb
});
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _aJs = __webpack_require__("./a.js");
_aJs.a;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);