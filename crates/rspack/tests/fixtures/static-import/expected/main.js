(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./b.js");
console.log('a');
},
"./b.js": function (module, exports, __webpack_require__) {
"use strict";
console.log('b');
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./a.js");
console.log('hello, world');
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);