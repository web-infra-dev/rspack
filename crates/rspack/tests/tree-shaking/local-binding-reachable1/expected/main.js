(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./Layout.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'defaults': function() { return defaults; }});
 const defaults = {
    test: 1000
};
},
"./export.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'callit': function() { return callit; }, 'Sider': function() { return Sider; }, 'Something': function() { return Something; }});
/* harmony import */var _Layout__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./Layout */"./Layout.js");

 function callit() {
    _Layout__WEBPACK_IMPORTED_MODULE__["defaults"].test;
}
 var Sider = callit();
 var Something = 20000;
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _export__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./export */"./export.js");

_export__WEBPACK_IMPORTED_MODULE__["Something"]();
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);