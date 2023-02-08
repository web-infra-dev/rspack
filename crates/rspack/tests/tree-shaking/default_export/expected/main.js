(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./answer.js": function (module, exports, __webpack_require__) {
"use strict";
__webpack_require__.d(exports, {
    "answer": ()=>answer
});
const answer = 103330;
},
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _libJs = __webpack_require__("./lib.js");
function render() {}
__webpack_require__.d(exports, {
    "result": ()=>result,
    "render": ()=>render
});
function result() {}
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _appJs = __webpack_require__.ir(__webpack_require__("./app.js"));
(0, _appJs.render)(_appJs.default);
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _answerJs = __webpack_require__("./answer.js");
const secret = "888";
__webpack_require__.d(exports, {
    "myanswer": ()=>myanswer,
    "secret": ()=>secret
});
const myanswer = _answerJs.answer;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);