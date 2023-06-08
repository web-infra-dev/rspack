(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'a': function() { return a; }});
/* harmony import */var _side_effects_js__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./side-effects.js */"./side-effects.js");
/* harmony import */var _side_effects_js__WEBPACK_IMPORTED_MODULE___default = /*#__PURE__*/__webpack_require__.n(_side_effects_js__WEBPACK_IMPORTED_MODULE__);


 const a = 3;
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _app__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./app */"./app.js");

_app__WEBPACK_IMPORTED_MODULE__["a"];
},
"./side-effects.js": function (module, exports, __webpack_require__) {
console.log("side effect");
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);