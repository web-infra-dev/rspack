(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./Something.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'something': function() { return something; }});
 function something() {}
},
"./export.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'default': function() { return a; }});
/* harmony import */var _Something__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./Something */"./Something.js");


var a = function test() {
    _Something__WEBPACK_IMPORTED_MODULE__["something"];
};

},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _export__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./export */"./export.js");

_export__WEBPACK_IMPORTED_MODULE__["default"];
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);