(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'default': function() { return abc; }
});
/* harmony import */var _dep_a__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./dep?a */"./dep.js?a");

function abc() {
    return _dep_a__WEBPACK_IMPORTED_MODULE_0_.x;
}
},
"./d.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'default': function() { return def; }
});
/* harmony import */var _dep_d__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./dep?d */"./dep.js?d");

class def {
    method() {
        return _dep_d__WEBPACK_IMPORTED_MODULE_0_.x;
    }
}
},
"./dep.js?a": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'x': function() { return x; },
  'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }
});
 const x = "x";
var __WEBPACK_DEFAULT_EXPORT__ = true;
},
"./dep.js?b": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }
});
 const x = "x";
var __WEBPACK_DEFAULT_EXPORT__ = false;
},
"./dep.js?c": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }
});
 const x = "x";
var __WEBPACK_DEFAULT_EXPORT__ = false;
},
"./dep.js?d": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'x': function() { return x; },
  'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }
});
 const x = "x";
var __WEBPACK_DEFAULT_EXPORT__ = true;
},
"./dep.js?e": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }
});
 const x = "x";
var __WEBPACK_DEFAULT_EXPORT__ = false;
},
"./dep.js?f": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }
});
 const x = "x";
var __WEBPACK_DEFAULT_EXPORT__ = false;
},
"./index.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _a__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./a */"./a.js");
/* harmony import */var _d__WEBPACK_IMPORTED_MODULE_3_ = __webpack_require__(/* ./d */"./d.js");
/* harmony import */var _dep_a__WEBPACK_IMPORTED_MODULE_6_ = __webpack_require__(/* ./dep?a */"./dep.js?a");
/* harmony import */var _dep_b__WEBPACK_IMPORTED_MODULE_7_ = __webpack_require__(/* ./dep?b */"./dep.js?b");
/* harmony import */var _dep_c__WEBPACK_IMPORTED_MODULE_8_ = __webpack_require__(/* ./dep?c */"./dep.js?c");
/* harmony import */var _dep_d__WEBPACK_IMPORTED_MODULE_9_ = __webpack_require__(/* ./dep?d */"./dep.js?d");
/* harmony import */var _dep_e__WEBPACK_IMPORTED_MODULE_10_ = __webpack_require__(/* ./dep?e */"./dep.js?e");
/* harmony import */var _dep_f__WEBPACK_IMPORTED_MODULE_11_ = __webpack_require__(/* ./dep?f */"./dep.js?f");












it("should generate valid code", ()=>{
    expect((0, _a__WEBPACK_IMPORTED_MODULE_0_["default"])()).toBe("x");
    expect(new (0, _d__WEBPACK_IMPORTED_MODULE_3_["default"])().method()).toBe("x");
});
it("a should be used", ()=>{
    expect(_dep_a__WEBPACK_IMPORTED_MODULE_6_["default"]).toBe(true);
});
it("b should be unused", ()=>{
    expect(_dep_b__WEBPACK_IMPORTED_MODULE_7_["default"]).toBe(false);
});
it("c should be used", ()=>{
    expect(_dep_c__WEBPACK_IMPORTED_MODULE_8_["default"]).toBe(true);
});
it("d should be used", ()=>{
    expect(_dep_d__WEBPACK_IMPORTED_MODULE_9_["default"]).toBe(true);
});
it("e should be unused", ()=>{
    expect(_dep_e__WEBPACK_IMPORTED_MODULE_10_["default"]).toBe(false);
});
it("f should be used", ()=>{
    expect(_dep_f__WEBPACK_IMPORTED_MODULE_11_["default"]).toBe(true);
});
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);