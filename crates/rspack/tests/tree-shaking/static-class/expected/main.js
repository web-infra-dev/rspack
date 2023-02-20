(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
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
var _bJs = __webpack_require__("./b.js");
class Test {
    static c = (0, _bJs.bb)();
    static test() {
        _bJs.bb;
    }
}
const a = 3;
},
"./b.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "bb", {
    enumerable: true,
    get: function() {
        return bb;
    }
});
const bb = 2;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _aJs = __webpack_require__("./a.js");
_aJs.a;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);