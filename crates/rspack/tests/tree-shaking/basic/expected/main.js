(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./answer.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "answer", {
    enumerable: true,
    get: function() {
        return answer;
    }
});
const answer = 103330;
},
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _libJs = __webpack_require__("./lib.js");
setTimeout(()=>{
    console.log(_libJs.myanswer);
}, 1000);
if (module.hot?.accept) module.hot.accept((module1)=>{
    console.log("xxx:", module1);
});
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./app.js");
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "myanswer", {
    enumerable: true,
    get: function() {
        return myanswer;
    }
});
var _answerJs = __webpack_require__("./answer.js");
const myanswer = _answerJs.answer;
},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);