(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./ b.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "b", {
    enumerable: true,
    get: function() {
        return b;
    }
});
const b = 3;
},
"./a.js": function (module, exports, __webpack_require__) {
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
const a = 3;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _aJs = __webpack_require__("./a.js");
var _bJs = __webpack_require__("./ b.js");
_aJs.a;
_bJs.b;
},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);