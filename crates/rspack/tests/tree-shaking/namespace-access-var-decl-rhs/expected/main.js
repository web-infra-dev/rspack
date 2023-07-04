(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'a': function() { return a; }});
 const a = {
    a: ''
};
},
"./b.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'b': function() { return b; }});
 const b = {
    b: ""
};
},
"./enum-old.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'a': function() { return _a__WEBPACK_IMPORTED_MODULE_0_["a"]; }});
__webpack_require__.d(__webpack_exports__, {'b': function() { return _b__WEBPACK_IMPORTED_MODULE_1_["b"]; }});
/* harmony import */var _a__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./a */"./a.js");
/* harmony import */var _b__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./b */"./b.js");


},
"./enum.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _enum_old__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./enum-old */"./enum-old.js");
__webpack_require__.es(_enum_old__WEBPACK_IMPORTED_MODULE_0_, __webpack_exports__);

},
"./index.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _lib__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./lib */"./lib.js");

console.log(_lib__WEBPACK_IMPORTED_MODULE_0_["getDocPermissionTextSendMe"]);
},
"./lib.js": function (module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {'getDocPermissionTextSendMe': function() { return getDocPermissionTextSendMe; }});
/* harmony import */var _enum_js__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./enum.js */"./enum.js");

function Record() {}
 const code2CreateChatDocPermission = {
    1: _enum_js__WEBPACK_IMPORTED_MODULE_0_.a.a
};
 function getDocPermissionTextSendMe() {}
 class Doc extends Record({}) {
    isSheet() {
        return this.type === _enum_js__WEBPACK_IMPORTED_MODULE_0_.b.b;
    }
}
Doc.fromJS = (data)=>new Doc(data);
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);