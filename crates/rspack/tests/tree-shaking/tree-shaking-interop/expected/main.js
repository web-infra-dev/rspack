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
__webpack_require__.e("bar_js").then(__webpack_require__.bind(__webpack_require__, "./bar.js")).then(__webpack_require__.ir).then((mod)=>{
    console.log(mod);
});
const a = "a";
exports.test = 30;
},
"./foo.js": function (module, exports, __webpack_require__) {
{
    const res = __webpack_require__("./a.js");
    module.exports = res;
}},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _fooJs = __webpack_require__.ir(__webpack_require__("./foo.js"));
(0, _fooJs.default)();
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);