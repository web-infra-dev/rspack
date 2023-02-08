(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "q", {
    enumerable: true,
    get: ()=>_libJs.question
});
const _libJs = __webpack_require__("./lib.js");
__webpack_require__.d(exports, {
    "q": ()=>_libJs.question
});
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _appJs = __webpack_require__("./app.js");
_appJs.q;
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "question": ()=>question
});
const question = "2";
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);