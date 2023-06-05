(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var pmodule_tracker__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* pmodule/tracker */"../node_modules/pmodule/tracker.js");
/* harmony import */var pmodule__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* pmodule */"../node_modules/pmodule/index.js");



pmodule__WEBPACK_IMPORTED_MODULE__["default"].should.be.eql("def");
pmodule__WEBPACK_IMPORTED_MODULE__["a"].should.be.eql("a");
pmodule__WEBPACK_IMPORTED_MODULE__["x"].should.be.eql("x");
pmodule__WEBPACK_IMPORTED_MODULE__["z"].should.be.eql("z");
pmodule_tracker__WEBPACK_IMPORTED_MODULE__["log"].should.be.eql([
    "a.js",
    "b.js",
    "c.js",
    "index.js"
]);
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);