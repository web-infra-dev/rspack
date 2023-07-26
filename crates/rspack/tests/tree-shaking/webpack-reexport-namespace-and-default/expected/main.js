(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _package1_script__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./package1/script */"./package1/script.js");
/* harmony import */var _package1_script2__WEBPACK_IMPORTED_MODULE_2_ = __webpack_require__(/* ./package1/script2 */"./package1/script2.js");
/* harmony import */var _package2_script__WEBPACK_IMPORTED_MODULE_3_ = __webpack_require__(/* ./package2/script */"./package2/script.js");



it("should load module correctly", ()=>{
    __webpack_require__(/* ./module */"./module.js");
});
// if (process.env.NODE_ENV === "production") {
it("default export should be unused", ()=>{
    expect(_package1_script__WEBPACK_IMPORTED_MODULE_1_["exportDefaultUsed"]).toBe(false);
    expect(_package1_script2__WEBPACK_IMPORTED_MODULE_2_["exportDefaultUsed"]).toBe(false);
});
// }
it("default export should be used", ()=>{
    expect(_package2_script__WEBPACK_IMPORTED_MODULE_3_["exportDefaultUsed"]).toBe(true);
});
},
"./module.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'mod': function() { return mod; }
});
/* harmony import */var _package1_script__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./package1/script */"./package1/script.js");
/* harmony import */var _package2_script__WEBPACK_IMPORTED_MODULE_1_ = __webpack_require__(/* ./package2/script */"./package2/script.js");


 const mod = _package2_script__WEBPACK_IMPORTED_MODULE_1_["default"];
},
"./package1/script.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'exportDefaultUsed': function() { return exportDefaultUsed; }
});
/* harmony import */var _script1__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./script1 */"./package1/script1.js");
/* harmony import */var _script1__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./script1 */"./package1/script1.js");
__webpack_require__.es(_script1__WEBPACK_IMPORTED_MODULE_0_, __webpack_exports__);

var __WEBPACK_DEFAULT_EXPORT__ = /* "./script1" unused */null;

 const exportDefaultUsed = __webpack_exports_info__.default.used;
},
"./package1/script1.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _script2__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./script2 */"./package1/script2.js");
__webpack_require__.es(_script2__WEBPACK_IMPORTED_MODULE_0_, __webpack_exports__);

var __WEBPACK_DEFAULT_EXPORT__ = 1;
},
"./package1/script2.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'exportDefaultUsed': function() { return exportDefaultUsed; }
});

function __WEBPACK_DEFAULT_EXPORT__(){
    return /* "./script3" unused */null;
}

 const exportDefaultUsed = __webpack_exports_info__.default.used;
},
"./package2/script.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'default': function() { return __WEBPACK_DEFAULT_EXPORT__; },
  'exportDefaultUsed': function() { return exportDefaultUsed; }
});
/* harmony import */var _script1__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./script1 */"./package2/script1.js");
/* harmony import */var _script1__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./script1 */"./package2/script1.js");
__webpack_require__.es(_script1__WEBPACK_IMPORTED_MODULE_0_, __webpack_exports__);

var __WEBPACK_DEFAULT_EXPORT__ = _script1__WEBPACK_IMPORTED_MODULE_0_["default"];

 const exportDefaultUsed = __webpack_exports_info__.default.used;
},
"./package2/script1.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }
});
var __WEBPACK_DEFAULT_EXPORT__ = 1;
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);