(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "app", {
    enumerable: true,
    get: ()=>app
});
const _lib = __webpack_require__("./lib.js");
var app = function() {
    _lib.result;
};
(0, _lib.something)('app4');
(0, _lib.something)('app3');
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _app = __webpack_require__("./app.js");
(0, _app.app)();
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
    result: ()=>result,
    something: ()=>something
});
const result = 20000;
const something = function() {};
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);