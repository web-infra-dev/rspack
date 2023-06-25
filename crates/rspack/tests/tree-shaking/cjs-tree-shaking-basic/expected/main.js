(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./answer.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'answer': function() { return answer; }});

 const answer = 42;
},
"./app.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'myanswer': function() { return _lib__WEBPACK_IMPORTED_MODULE__["myanswer"]; }});
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./lib */"./lib.js");

},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _app__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./app */"./app.js");

__webpack_require__(/* ./answer */"./answer.js");
(0, _app__WEBPACK_IMPORTED_MODULE__["myanswer"])();
},
"./lib.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'myanswer': function() { return myanswer; }});
 const myanswer = 'anyser';
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);