(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "q", {
    enumerable: true,
    get: function() {
        return _libJs.question;
    }
});
var _libJs = __webpack_require__("./lib.js");
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _appJs = __webpack_require__("./app.js");
_appJs.q;
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "question", {
    enumerable: true,
    get: function() {
        return question;
    }
});
const question = "2";
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);