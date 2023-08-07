(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./answer.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'answer': function() { return answer; }
});
 const answer = 103330; // export default answer;
},
"./app.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'render': function() { return render; },
  'default': function() { return result; }
});
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./lib */"./lib.js");

 function render() {
    function test() {
        const container = document.getElementById("root");
        container.innerHTML = `adddd333:${_lib__WEBPACK_IMPORTED_MODULE_0_.secret}:${_lib__WEBPACK_IMPORTED_MODULE_0_.myanswer}`;
    }
}
function result() {}
},
"./index.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _app__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./app */"./app.js");

(0, _app__WEBPACK_IMPORTED_MODULE_0_.render)(_app__WEBPACK_IMPORTED_MODULE_0_["default"]);
},
"./lib.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'secret': function() { return secret; },
  'myanswer': function() { return myanswer; }
});
/* harmony import */var _answer__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./answer */"./answer.js");

 const secret = "888";
 const myanswer = _answer__WEBPACK_IMPORTED_MODULE_0_.answer, result = 20000;
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);