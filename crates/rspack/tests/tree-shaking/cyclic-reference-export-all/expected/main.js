(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./src/App.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }
});
/* harmony import */var _containers__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./containers */"./src/containers/index.js");

const { PlatformProvider } = _containers__WEBPACK_IMPORTED_MODULE_0_.containers;
const Index = ()=>{
    console.log("PlatformProvider", PlatformProvider);
    return 'something';
};
var __WEBPACK_DEFAULT_EXPORT__ = Index;
},
"./src/containers/containers.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _platform_container__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./platform-container */"./src/containers/platform-container/index.js");
__webpack_require__.es(_platform_container__WEBPACK_IMPORTED_MODULE_0_, __webpack_exports__);


},
"./src/containers/index.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'containers': function() { return _containers__WEBPACK_IMPORTED_MODULE_0_; }
});
/* harmony import */var _containers__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./containers */"./src/containers/containers.js");


},
"./src/containers/platform-container/index.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  'usePlatform': function() { return usePlatform; },
  'PlatformProvider': function() { return PlatformProvider; }
});
 const usePlatform = 3;
 const PlatformProvider = 1000;
},
"./src/index.js": function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
'use strict';
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _App__WEBPACK_IMPORTED_MODULE_0_ = __webpack_require__(/* ./App */"./src/App.js");

(0, _App__WEBPACK_IMPORTED_MODULE_0_["default"])();
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./src/index.js"));

}
]);