(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./Layout.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "defaults", {
    enumerable: true,
    get: ()=>defaults
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
Object.defineProperty(exports, "Something", {
    enumerable: true,
    get: ()=>Something
});
const _layout = __webpack_require__("./Layout.js");
class Test {
    test = _layout.defaults.test + 20000;
}
new Test();
var Something = 333;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _export = __webpack_require__("./export.js");
(0, _export.Something)();
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);