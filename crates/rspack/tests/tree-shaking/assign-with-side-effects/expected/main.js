(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _libJs = __webpack_require__("./lib.js");
__webpack_require__.d(exports, {
    "app": ()=>app
});
function app() {}
app.prototype.result = _libJs.result;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _appJs = __webpack_require__("./app.js");
(0, _appJs.app)();
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "result": ()=>result
});
const result = 20000;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);