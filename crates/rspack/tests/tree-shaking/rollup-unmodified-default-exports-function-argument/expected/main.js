(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./foo.js": function (module, exports, __webpack_require__) {
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
    default: function() {
        return _default;
    },
    bar: function() {
        return bar;
    }
});
var foo = function() {
    return 42;
};
var _default = foo;
function bar() {
    return contrivedExample(foo);
}
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _fooJs = __webpack_require__.ir(__webpack_require__("./foo.js"));
var answer = (0, _fooJs.default)();
(0, _fooJs.bar)();
console.log(answer);
},

},function(__webpack_require__) {
var __webpack_exports__ = __webpack_require__('./index.js');

}
]);