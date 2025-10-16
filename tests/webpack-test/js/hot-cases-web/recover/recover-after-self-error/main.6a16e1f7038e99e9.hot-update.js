"use strict";
self["webpackHotUpdate"]("main", {
"./a.js": 
/*!**************!*\
  !*** ./a.js ***!
  \**************/
(function (module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  "default": () => (__WEBPACK_DEFAULT_EXPORT__),
  getError: () => (getError),
  id: () => (id)
});
module.hot.data.store.error = false;
module.hot.data.store.value = 4;
/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (() => { throw new Error("should not happen") });
const getError = () => { throw new Error("should not happen") };
const id = module.id;


}),

},function(__webpack_require__) {
// webpack/runtime/define_property_getters
(() => {
__webpack_require__.d = (exports, definition) => {
	for(var key in definition) {
        if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
            Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
        }
    }
};
})();
// webpack/runtime/get_full_hash
(() => {
__webpack_require__.h = () => ("f8594caaeda3f8c2")
})();

}
);