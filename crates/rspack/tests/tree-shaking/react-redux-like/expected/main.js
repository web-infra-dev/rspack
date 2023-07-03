(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'Provider': function() { return _lib__WEBPACK_IMPORTED_MODULE_0_["default"]; }});
__webpack_require__.d(__webpack_exports__, {'useSelector': function() { return _selector_js__WEBPACK_IMPORTED_MODULE_1_["default"]; }});
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./lib */"./lib.js");
/* harmony import */var _selector_js__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./selector.js */"./selector.js");



},
"./foo.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _app__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./app */"./app.js");
__webpack_require__.es(_app__WEBPACK_IMPORTED_MODULE_0_, __webpack_exports__);

function batch() {}

},
"./index.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _foo__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./foo */"./foo.js");

_foo__WEBPACK_IMPORTED_MODULE_0_["Provider"];
_foo__WEBPACK_IMPORTED_MODULE_0_["useSelector"];
},
"./lib.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }});
function Provider() {}
var __WEBPACK_DEFAULT_EXPORT__ = Provider;
},
"./selector.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'default': function() { return useSelector; }});
function useSelector() {
    return "";
}
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);