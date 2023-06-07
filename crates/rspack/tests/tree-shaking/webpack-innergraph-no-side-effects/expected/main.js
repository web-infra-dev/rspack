(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
it("should be able to load package without side effects where modules are unused", ()=>{
    __webpack_require__(/* ./module */"./module.js");
});
},
"./module.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'test': function() { return test; }});
/* harmony import */var _package__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./package */"./package/index.js");

var __WEBPACK_DEFAULT_EXPORT__ = _package__WEBPACK_IMPORTED_MODULE__["a"];
 function test() {}
},
"./package/index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'a': function() { return a; }});
/* harmony import */var _unusedModule__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./unusedModule */"./package/unusedModule.js");

 function a() {
    return 42;
}
 function b() {
    return _unusedModule__WEBPACK_IMPORTED_MODULE__["default"];
}
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);