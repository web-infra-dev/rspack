(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "answer", {
    enumerable: true,
    get: ()=>answer
});
const answer = 30;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _appJs = __webpack_require__("./app.js");
const a = test(()=>__webpack_require__.e("lib_js").then(__webpack_require__.bind(__webpack_require__, "./lib.js")).then(__webpack_require__.ir));
(0, _appJs.answer)();
a;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);