(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./zh_locale.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
exports["default"] = void 0;
/* eslint-disable no-template-curly-in-string */ var _default = {};
exports["default"] = _default;
},
"./antd/index.ts": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'locales': function() { return locales; }});
/* harmony import */var _locale_zh__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ../locale_zh */"./locale_zh.ts");

const locales = {
    zh_CN
};

},
"./index.ts": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'test': function() { return test; }});
/* harmony import */var _antd_index__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./antd/index */"./antd/index.ts");

_antd_index__WEBPACK_IMPORTED_MODULE__["locales"].zh_CN;
 function test() {}
},
"./locale_zh.ts": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {});
/* harmony import */var _zh_locale__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./zh_locale */"./zh_locale.js");

var __WEBPACK_DEFAULT_EXPORT__ = _zh_locale__WEBPACK_IMPORTED_MODULE__;
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.ts"));

}
]);