(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./answer.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'answer': function() { return answer; }});
 const answer = 103330;
},
"./app.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./lib */"./lib.js");

setTimeout(()=>{
    console.log(_lib__WEBPACK_IMPORTED_MODULE__["myanswer"]);
}, 1000);
 function render() {
    function test() {
        const container = document.getElementById("root");
        container.innerHTML = `adddd333:${_lib__WEBPACK_IMPORTED_MODULE__["secret"]}:${_lib__WEBPACK_IMPORTED_MODULE__["myanswer"]}`;
    }
}
if (module.hot?.accept) module.hot.accept((module)=>{
    console.log("xxx:", module);
});
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _app__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./app */"./app.js");

},
"./lib.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'myanswer': function() { return myanswer; }});
/* harmony import */var _answer__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./answer */"./answer.js");

 const secret = "888";
 const myanswer = _answer__WEBPACK_IMPORTED_MODULE__["answer"], result = 20000;
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);