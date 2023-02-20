(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./zh_locale.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
exports["default"] = void 0;
var _default = {};
exports["default"] = _default;
},
"./antd/index.ts": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "locales", {
    enumerable: true,
    get: function() {
        return locales;
    }
});
var _localeZhTs = __webpack_require__.ir(__webpack_require__("./locale_zh.ts"));
const locales = {
    zh_CN: _localeZhTs.default
};
},
"./index.ts": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "test", {
    enumerable: true,
    get: function() {
        return test;
    }
});
var _indexTs = __webpack_require__("./antd/index.ts");
_indexTs.locales.zh_CN;
function test() {}
},
"./locale_zh.ts": function (module, exports, __webpack_require__) {
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
var _zhLocaleJs = __webpack_require__.ir(__webpack_require__("./zh_locale.js"));
var _default = _zhLocaleJs.default;
},

},function(__webpack_require__) {
__webpack_require__("./index.ts");
}
]);