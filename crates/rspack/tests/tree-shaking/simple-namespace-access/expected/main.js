(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _mathsJs = __webpack_require__.ir(__webpack_require__("./maths.js"));
console.log(_mathsJs.xxx.test);
console.log(_mathsJs['square']);
},
"./maths.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "xxx", {
    enumerable: true,
    get: ()=>_testJs
});
const _testJs = __webpack_require__.ir(__webpack_require__("./test.js"));
function square(x) {
    return x * x;
}
__webpack_require__.d(exports, {
    "square": ()=>square
});
},
"./test.js": function (module, exports, __webpack_require__) {
"use strict";
function test() {}
__webpack_require__.d(exports, {
    "test": ()=>test,
    "ccc": ()=>ccc
});
function ccc() {}
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);