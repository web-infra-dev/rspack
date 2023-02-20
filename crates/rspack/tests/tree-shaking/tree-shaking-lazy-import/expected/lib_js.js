(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["lib_js"], {
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _testJs = __webpack_require__.ir(__webpack_require__("./test.js"));
function myanswer() {
    _testJs.default;
}
var _default = myanswer;
},
"./test.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function test() {}
var _default = test;
},

}]);