(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./Layout.js": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "defaults": ()=>defaults
});
const defaults = {
    test: 1000
};
},
"./export.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _layoutJs = __webpack_require__("./Layout.js");
function callit() {
    _layoutJs.defaults.test;
}
callit();
__webpack_require__.d(exports, {
    "Something": ()=>Something
});
var Something = 20000;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _exportJs = __webpack_require__("./export.js");
(0, _exportJs.Something)();
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);