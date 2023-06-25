(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'a': function() { return a; }});
__webpack_require__.el(/* ./bar */"./bar.js").then(__webpack_require__.bind(__webpack_require__, /* ./bar */"./bar.js")).then((mod)=>{
    console.log(mod);
});
 const a = "a";
exports.test = 30;
},
"./foo.js": function (module, exports, __webpack_require__) {
{
    const res = __webpack_require__(/* ./a */"./a.js");
    module.exports = res;
} // export default function () {}
},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _foo__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./foo */"./foo.js");
/* harmony import */var _foo__WEBPACK_IMPORTED_MODULE___default = /*#__PURE__*/__webpack_require__.n(_foo__WEBPACK_IMPORTED_MODULE__);

_foo__WEBPACK_IMPORTED_MODULE___default();
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);