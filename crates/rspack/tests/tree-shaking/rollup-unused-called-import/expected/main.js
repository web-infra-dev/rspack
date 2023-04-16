(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./dead.js": function (module, exports, __webpack_require__) {
"use strict";
},
"./foo.js": function (module, exports, __webpack_require__) {
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
__webpack_require__("./dead.js");
function _default() {
    return "foo";
}
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _foo = __webpack_require__.ir(__webpack_require__("./foo.js"));
assert.equal((0, _foo.default)(), "foo");
},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);