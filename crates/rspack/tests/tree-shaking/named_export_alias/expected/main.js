(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./Something.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "something", {
    enumerable: true,
    get: ()=>something
});
function something() {}
},
"./export.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>a
});
const _something = __webpack_require__("./Something.js");
var a = function test() {
    _something.something;
};
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _export = __webpack_require__.interopRequire(__webpack_require__("./export.js"));
_export.default;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);