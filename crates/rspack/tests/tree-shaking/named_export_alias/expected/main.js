(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./Something.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'something': function() { return something; }});
 function something() {}
},
"./export.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'default': function() { return a; }});
/* harmony import */var _Something__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./Something */"./Something.js");


var a = function test() {
    _Something__WEBPACK_IMPORTED_MODULE_1_["something"];
};

},
"./index.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _export__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./export */"./export.js");

_export__WEBPACK_IMPORTED_MODULE_0_["default"];
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);