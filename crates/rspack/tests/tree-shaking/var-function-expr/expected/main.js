(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _libJs = __webpack_require__("./lib.js");
var app = function() {
    _libJs.result;
};
__webpack_require__.d(exports, {
    "app": ()=>app
});
(0, _libJs.something)('app4');
(0, _libJs.something)('app3');
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
    "something": ()=>something,
    "result": ()=>result
});
const result = 20000;
const something = function() {};
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);