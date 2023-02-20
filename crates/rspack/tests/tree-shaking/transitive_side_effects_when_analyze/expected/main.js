(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: function() {
        return a;
    }
});
__webpack_require__("./side-effects.js");
const a = 3;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _appJs = __webpack_require__("./app.js");
_appJs.a;
},
"./side-effects.js": function (module, exports, __webpack_require__) {
console.log("side effect");
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);