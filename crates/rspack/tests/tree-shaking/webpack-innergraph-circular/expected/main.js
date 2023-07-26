(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _inner__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./inner */"./inner.js");
/* harmony import */var _module__WEBPACK_IMPORTED_MODULE_2_ = __webpack_require__(/* ./module */"./module.js");


it("export should be unused when only unused functions use it", ()=>{
    expect((0, _module__WEBPACK_IMPORTED_MODULE_2_["y"])("a")).toBe("okBAA");
    expect(_inner__WEBPACK_IMPORTED_MODULE_1_["exportAUsed"]).toBe(true);
    expect(_inner__WEBPACK_IMPORTED_MODULE_1_["exportBUsed"]).toBe(true);
    expect(_inner__WEBPACK_IMPORTED_MODULE_1_["exportCUsed"]).toBe(false);
    return __webpack_require__.el(/* ./chunk */"./chunk.js").then(__webpack_require__.bind(__webpack_require__, /* ./chunk */"./chunk.js"));
});
},
"./inner.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'A': function() { return A; },
  'B': function() { return B; },
  'exportAUsed': function() { return exportAUsed; },
  'exportBUsed': function() { return exportBUsed; },
  'exportCUsed': function() { return exportCUsed; }
});
 function A(s) {
    return s + "A";
}
 function B(s) {
    return s + "B";
}
 function C(s) {
    return s + "C";
}
 const exportAUsed = __webpack_exports_info__.A.used;
 const exportBUsed = __webpack_exports_info__.B.used;
 const exportCUsed = __webpack_exports_info__.C.used;
},
"./module.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'y': function() { return y; }
});
/* harmony import */var _inner__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./inner */"./inner.js");

function x(type) {
    switch(type){
        case "a":
            return withA("b");
        case "b":
            return withB("c");
        case "c":
            return "ok";
    }
}
function y(v) {
    return withA(v);
}
function withA(v) {
    const value = x(v);
    return (0, _inner__WEBPACK_IMPORTED_MODULE_0_["A"])(value);
}
function withB(v) {
    const value = x(v);
    return (0, _inner__WEBPACK_IMPORTED_MODULE_0_["B"])(value);
}
function withC(v) {
    const value = x(v);
    return /* "./inner" unused */null(value);
}

},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);