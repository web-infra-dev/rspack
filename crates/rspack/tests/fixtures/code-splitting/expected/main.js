(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
console.log('hello, world');
__webpack_require__.e("a_js").then(__webpack_require__.bind(__webpack_require__, "./a.js"));
__webpack_require__.e("b_js").then(__webpack_require__.bind(__webpack_require__, "./b.js"));
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);