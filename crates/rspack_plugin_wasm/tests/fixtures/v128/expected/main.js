(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.a(module, async function (__webpack_handle_async_dependencies__, __webpack_async_result__) { try {
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _v128Wasm = __webpack_require__.ir(__webpack_require__("./v128.wasm"));
var __webpack_async_dependencies__ = __webpack_handle_async_dependencies__([
    _v128Wasm
]);
[_v128Wasm] = __webpack_async_dependencies__.then ? (await __webpack_async_dependencies__)() : __webpack_async_dependencies__;
console.log(_v128Wasm.default.x);

__webpack_async_result__();
} catch(e) { __webpack_async_result__(e); } });},
"./v128.wasm": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.a(module, async function (__webpack_handle_async_dependencies__, __webpack_async_result__) { try {
 module.exports = __webpack_require__.v(exports, module.id, "c61e7cc882ba31f8.module.wasm" );
__webpack_async_result__();
} catch(e) { __webpack_async_result__(e); } });},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);