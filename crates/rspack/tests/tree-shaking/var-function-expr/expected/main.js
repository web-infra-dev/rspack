(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'app': function() { return app; }});
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./lib */"./lib.js");

 var app = function() {
    _lib__WEBPACK_IMPORTED_MODULE__["result"];
};
 var app2 = ()=>{
    _lib__WEBPACK_IMPORTED_MODULE__["secret"];
};
 var app4 = (0, _lib__WEBPACK_IMPORTED_MODULE__["something"])('app4'), app5 = 10000;
 var app3 = (0, _lib__WEBPACK_IMPORTED_MODULE__["something"])('app3');
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _app__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./app */"./app.js");

(0, _app__WEBPACK_IMPORTED_MODULE__["app"])();
},
"./lib.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'result': function() { return result; }, 'something': function() { return something; }});
 const secret = "888";
 const result = 20000;
 const something = function() {};
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);