(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _mathsJs = __webpack_require__.ir(__webpack_require__("./maths.js"));
console.log(_mathsJs.xxx.test);
console.log(_mathsJs['square']);
},
"./maths.js": function (module, exports, __webpack_require__) {
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
    square: function() {
        return square;
    },
    xxx: function() {
        return _testJs;
    }
});
var _testJs = __webpack_require__.ir(__webpack_require__("./test.js"));
function square(x) {
    return x * x;
}
},
"./test.js": function (module, exports, __webpack_require__) {
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
    test: function() {
        return test;
    },
    ccc: function() {
        return ccc;
    }
});
function test() {}
function ccc() {}
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);