(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"../node_modules/side-effects-module/index.js": function (module, exports, __webpack_require__) {
"use strict";
},
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.exportStar(__webpack_require__("./lib.js"), exports);
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _app = __webpack_require__("./app.js");
(0, _app.something)();
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "something", {
    enumerable: true,
    get: ()=>something
});
const something = function() {};
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);