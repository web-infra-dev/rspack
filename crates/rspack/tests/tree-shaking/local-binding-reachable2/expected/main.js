(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./Layout.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "defaults", {
    enumerable: true,
    get: function() {
        return defaults;
    }
});
const defaults = {
    test: 1000
};
},
"./export.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "Something", {
    enumerable: true,
    get: function() {
        return Something;
    }
});
var _layoutJs = __webpack_require__("./Layout.js");
class Test {
    test = _layoutJs.defaults.test + 20000;
}
new Test();
var Something = 333;
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _exportJs = __webpack_require__("./export.js");
(0, _exportJs.Something)();
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);