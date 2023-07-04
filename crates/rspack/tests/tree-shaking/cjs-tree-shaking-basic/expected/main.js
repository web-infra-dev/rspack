(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./answer.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'answer': function() { return answer; }});

 const answer = 42;
},
"./app.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'myanswer': function() { return _lib__WEBPACK_IMPORTED_MODULE_0_["myanswer"]; }});
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./lib */"./lib.js");

},
"./index.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _app__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./app */"./app.js");

__webpack_require__(/* ./answer */"./answer.js");
(0, _app__WEBPACK_IMPORTED_MODULE_1_["myanswer"])();
},
"./lib.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'myanswer': function() { return myanswer; }});
 const myanswer = 'anyser';
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);