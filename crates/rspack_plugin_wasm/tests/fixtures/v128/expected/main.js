(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
__webpack_require__.a(module, async function(__webpack_handle_async_dependencies__, __webpack_async_result__) {
    try {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        var _v128wasm = __webpack_require__.ir(__webpack_require__("./v128.wasm"));
        var __webpack_async_dependencies__ = __webpack_handle_async_dependencies__([
            _v128wasm
        ]);
        [_v128wasm] = __webpack_async_dependencies__.then ? (await __webpack_async_dependencies__)() : __webpack_async_dependencies__;
        console.log(_v128wasm.default.x);
        __webpack_async_result__();
    } catch (e) {
        __webpack_async_result__(e);
    }
});
},
"./v128.wasm": function (module, exports, __webpack_require__) {
 module.exports = __webpack_require__.v(exports, module.id, "93ae28133776ccef.module.wasm" );},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);