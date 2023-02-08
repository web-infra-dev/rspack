(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./bar.js": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "test": ()=>test
});
function test() {}
},
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "Select", {
    enumerable: true,
    get: ()=>_barJs.default
});
const _barJs = __webpack_require__.ir(__webpack_require__("./bar.js"));
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _fooJs = __webpack_require__("./foo.js");
(0, _fooJs.Select)();
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);