(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["dynamic-1_js"], {
"./dynamic-1.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./shared.js");
__webpack_require__.el("./dynamic-2.js").then(__webpack_require__.bind(__webpack_require__, "./dynamic-2.js")).then(__webpack_require__.ir);
console.log('dynamic-1');
},
"./shared.js": function (module, exports, __webpack_require__) {
console.log('shared');
},

}]);