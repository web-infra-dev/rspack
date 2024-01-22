"use strict";
self["webpackHotUpdate"]('main', {
"./child.jsx": (function (module, __webpack_exports__, __webpack_require__) {
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  Child: function() { return Child; }
});
/* harmony import */var react_jsx_dev_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-dev-runtime */ "../../../../../../node_modules/preact/compat/jsx-dev-runtime.js");
/* module decorator */ module = __webpack_require__.hmd(module);
/* provided dependency */ var __prefresh_utils__ = __webpack_require__(/*! ../../../../client/prefresh.js */ "../../../../client/prefresh.js");

function Child() {
    return /*#__PURE__*/ (0,react_jsx_dev_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxDEV)("span", {
        children: "child change"
    }, void 0, false, {
        fileName: "/Users/bytedance/rspack-dev/rspack/packages/rspack-plugin-preact-refresh/tests/hotCases/jsx/child/child.jsx",
        lineNumber: 2,
        columnNumber: 11
    }, this);
}
_c = Child;
var _c;
$RefreshReg$(_c, "Child");

/**
 * The following code is modified based on
 * //https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/packages/webpack/src/loader/runtime.js
 *
 * MIT Licensed
 * Author JoviDeCroock
 * Copyright (c) 2021-Present Preact Team
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/LICENSE
 */

const isPrefreshComponent = __prefresh_utils__.shouldBind(module);

// `@vanilla-extract/webpack` does some custom preprocessing where
// `module.hot` is partially replaced. This leads to our injected
// code being executed although it shouldn't be:
//
// Intermediate result:
//
//   if (true) { // <- inlined by intermediate compile step
//     const previousHotModuleExports = module.hot.data && ...
//                    // Crash happens here ---^
//
// It crashes at that line because some intermediate compiler isn't
// running in hot mode, but the overall guard condition was compiled
// down to being truthy. By moving `module.hot` outside of the
// condition of the if-statement, it will be left as is.
const moduleHot = module.hot;

if (moduleHot) {
  const currentExports = __prefresh_utils__.getExports(module);
  const previousHotModuleExports =
    moduleHot.data && moduleHot.data.moduleExports;

  __prefresh_utils__.registerExports(currentExports, module.id);

  if (isPrefreshComponent) {
    if (previousHotModuleExports) {
      try {
        __prefresh_utils__.flush();
        if (
          typeof __prefresh_errors__ !== 'undefined' &&
          __prefresh_errors__ &&
          __prefresh_errors__.clearRuntimeErrors
        ) {
          __prefresh_errors__.clearRuntimeErrors();
        }
      } catch (e) {
        // Only available in newer webpack versions.
        if (moduleHot.invalidate) {
          moduleHot.invalidate();
        } else {
          self.location.reload();
        }
      }
    }

    moduleHot.dispose(data => {
      data.moduleExports = __prefresh_utils__.getExports(module);
    });

    moduleHot.accept(function errorRecovery() {
      if (
        typeof __prefresh_errors__ !== 'undefined' &&
        __prefresh_errors__ &&
        __prefresh_errors__.handleRuntimeError
      ) {
        __prefresh_errors__.handleRuntimeError(error);
      }

      __webpack_require__.c[module.id].hot.accept(errorRecovery);
    });
  }
}

}),

},function(__webpack_require__) {
// webpack/runtime/get_full_hash
(() => {
__webpack_require__.h = function () {
	return "5d8bbf6fece35c2a6eaf";
};

})();

}
);