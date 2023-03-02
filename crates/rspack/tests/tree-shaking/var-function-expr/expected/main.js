(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "app", {
    enumerable: true,
    get: function() {
        return app;
    }
});
var _libJs = __webpack_require__("./lib.js");
var app = function() {
    _libJs.result;
};
(0, _libJs.something)('app4');
(0, _libJs.something)('app3');
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _appJs = __webpack_require__("./app.js");
(0, _appJs.app)();
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
    result: function() {
        return result;
    },
    something: function() {
        return something;
    }
});
const result = 20000;
const something = function() {};
},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);