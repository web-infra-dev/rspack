(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./dead.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
function __WEBPACK_DEFAULT_EXPORT__(){
    return "dead";
}
},
"./foo.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }
});
/* harmony import */var _dead__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./dead */"./dead.js");

function __WEBPACK_DEFAULT_EXPORT__(){
    return "foo";
}
 function foodead() {
    return "foo" + (0, _dead__WEBPACK_IMPORTED_MODULE_0_["default"])();
}
},
"./index.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _foo__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./foo */"./foo.js");

assert.equal((0, _foo__WEBPACK_IMPORTED_MODULE_0_["default"])(), "foo");
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);