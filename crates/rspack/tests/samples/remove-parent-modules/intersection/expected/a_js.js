(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["a_js"], {
"./a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./shared.js");
__webpack_require__("./i-1.js");
__webpack_require__("./i-2.js");
console.log('a');
},
"./i-1.js": function (module, exports, __webpack_require__) {
console.log('i-1');
},
"./i-2.js": function (module, exports, __webpack_require__) {
console.log('i-2');
},

}]);