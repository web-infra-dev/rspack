(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./export.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "Sider", {
    enumerable: true,
    get: ()=>Sider
});
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _exportJs = __webpack_require__("./export.js");
(0, _exportJs.Sider)();
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);