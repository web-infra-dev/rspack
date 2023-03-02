(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./b.js?x": function (module, exports, __webpack_require__) {
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
var _bJsx = __webpack_require__("./b.jsx");
var _bJsX = __webpack_require__("./b.js?x");
_bJsX.a;
_bJsx.a;
console.log("hello, world");
},
"./b.jsx": function (module, exports, __webpack_require__) {
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

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);