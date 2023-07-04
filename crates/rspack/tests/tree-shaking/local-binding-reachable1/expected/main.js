(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./Layout.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'defaults': function() { return defaults; }});
 const defaults = {
    test: 1000
};
},
"./export.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'Something': function() { return Something; }});
/* harmony import */var _Layout__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./Layout */"./Layout.js");

 function callit() {
    _Layout__WEBPACK_IMPORTED_MODULE_0_["defaults"].test;
}
 var Sider = callit();
 var Something = 20000;
},
"./index.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _export__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./export */"./export.js");

(0, _export__WEBPACK_IMPORTED_MODULE_0_["Something"])();
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);