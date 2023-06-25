(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./Something.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'default': function() { return Something; }});
class Something {
}
},
"./export.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'Sider': function() { return _Something__WEBPACK_IMPORTED_MODULE__["default"]; }});
/* harmony import */var _Something__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./Something */"./Something.js");



},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _export__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./export */"./export.js");

(0, _export__WEBPACK_IMPORTED_MODULE__["Sider"])();
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);