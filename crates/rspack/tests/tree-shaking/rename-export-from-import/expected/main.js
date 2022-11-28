(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "q", {
    enumerable: true,
    get: ()=>_lib.question
});
const _lib = __webpack_require__("./lib.js");
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _app = __webpack_require__("./app.js");
_app.q;
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "question", {
    enumerable: true,
    get: ()=>question
});
const question = "2";
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);