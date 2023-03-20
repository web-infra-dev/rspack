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
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    render: function() {
        return render;
    },
    default: function() {
        return result;
    }
});
__webpack_require__("./lib.js");
function render() {}
function result() {}
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _appJs = __webpack_require__.ir(__webpack_require__("./app.js"));
(0, _appJs.render)(_appJs.default);
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
    secret: function() {
        return secret;
    },
    myanswer: function() {
        return myanswer;
    }
});
var _answerJs = __webpack_require__("./answer.js");
const secret = "888";
const myanswer = _answerJs.answer;
},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);