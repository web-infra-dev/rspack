var __webpack_modules__ = ({
"./style.css?eccb": (function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  aClass: function() { return aClass; },
  bClass: function() { return bClass; },
  cClass: function() { return cClass; }
});
// extracted by css-extract-rspack-plugin
var aClass = "foo__style__a-class";
var bClass = "foo__style__b__class";
var cClass = "foo__style__cClass";
}),

});
/************************************************************************/
// The module cache
var __webpack_module_cache__ = {};

// The require function
function __webpack_require__(moduleId) {

// Check if module is in cache
var cachedModule = __webpack_module_cache__[moduleId];
if (cachedModule !== undefined) {
return cachedModule.exports;
}
// Create a new module (and put it into the cache)
var module = (__webpack_module_cache__[moduleId] = {
exports: {}
});
// Execute the module function
__webpack_modules__[moduleId](module, module.exports, __webpack_require__);

// Return the exports of the module
return module.exports;

}

/************************************************************************/
// webpack/runtime/define_property_getters
(() => {
__webpack_require__.d = function(exports, definition) {
	for(var key in definition) {
        if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
            Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
        }
    }
};
})();
// webpack/runtime/has_own_property
(() => {
__webpack_require__.o = function (obj, prop) {
	return Object.prototype.hasOwnProperty.call(obj, prop);
};

})();
// webpack/runtime/make_namespace_object
(() => {
// define __esModule on exports
__webpack_require__.r = function(exports) {
	if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
		Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
	}
	Object.defineProperty(exports, '__esModule', { value: true });
};

})();
/************************************************************************/
var __webpack_exports__ = {};
// This entry need to be wrapped in an IIFE because it need to be isolated against other modules in the chunk.
(() => {
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _style_css__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__("./style.css?eccb");

// eslint-disable-next-line no-console
console.log({
    css: _style_css__WEBPACK_IMPORTED_MODULE_0__["default"],
    aClass: _style_css__WEBPACK_IMPORTED_MODULE_0__.aClass,
    bClass: _style_css__WEBPACK_IMPORTED_MODULE_0__.bClass,
    cClass: _style_css__WEBPACK_IMPORTED_MODULE_0__.cClass
});

})();

