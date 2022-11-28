(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./answer.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "answer", {
    enumerable: true,
    get: ()=>answer
});
const answer = 103330;
},
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    render: ()=>render,
    default: ()=>result
});
function render() {}
function result() {}
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _app = __webpack_require__.interopRequire(__webpack_require__("./app.js"));
(0, _app.render)(_app.default);
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    secret: ()=>secret,
    myanswer: ()=>myanswer
});
const _answer = __webpack_require__("./answer.js");
const secret = "888";
const myanswer = _answer.answer;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);