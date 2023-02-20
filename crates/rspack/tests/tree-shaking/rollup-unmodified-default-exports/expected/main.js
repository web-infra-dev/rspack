(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var Foo = function() {
    console.log("side effect");
    this.isFoo = true;
};
var _default = Foo;
Foo.prototype = {
    answer: function() {
        return 42;
    }
};
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _fooJs = __webpack_require__.ir(__webpack_require__("./foo.js"));
new _fooJs.default();
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);