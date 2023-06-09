(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'a': function() { return a; }});
 const a = {
    a: ''
};
},
"./b.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'b': function() { return b; }});
 const b = {
    b: ""
};
},
"./enum-old.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'a': function() { return _a__WEBPACK_IMPORTED_MODULE__["a"]; }});
__webpack_require__.d(exports, {'b': function() { return _b__WEBPACK_IMPORTED_MODULE__["b"]; }});
/* harmony import */var _a__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./a */"./a.js");
/* harmony import */var _b__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./b */"./b.js");


},
"./enum.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _enum_old__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./enum-old */"./enum-old.js");
__webpack_require__.es(_enum_old__WEBPACK_IMPORTED_MODULE__, exports);

},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./lib */"./lib.js");

console.log(_lib__WEBPACK_IMPORTED_MODULE__["getDocPermissionTextSendMe"]);
},
"./lib.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'getDocPermissionTextSendMe': function() { return getDocPermissionTextSendMe; }});
/* harmony import */var _enum_js__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./enum.js */"./enum.js");

function Record() {}
 const code2CreateChatDocPermission = {
    1: _enum_js__WEBPACK_IMPORTED_MODULE__.a.a
};
 function getDocPermissionTextSendMe() {}
 class Doc extends Record({}) {
    isSheet() {
        return this.type === _enum_js__WEBPACK_IMPORTED_MODULE__.b.b;
    }
}
Doc.fromJS = (data)=>new Doc(data);
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);