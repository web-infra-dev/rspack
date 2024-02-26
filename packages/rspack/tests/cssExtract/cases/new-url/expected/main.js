(function () {
	var __webpack_modules__ = {
		"./style.css?778c": function (
			__unused_webpack_module,
			__webpack_exports__,
			__webpack_require__
		) {
			"use strict";
			__webpack_require__.r(__webpack_exports__);
			// extracted by rspack-mini-css-extract-plugin
		},
		"./index.js": function (
			__unused_webpack_module,
			__unused_webpack_exports,
			__webpack_require__
		) {
			const myURL = new URL(
				/* asset import */ __webpack_require__(
					/*! ./style.css */ "./style.css?778c"
				),
				__webpack_require__.b
			);
			console.log(myURL);
		}
	};
	// The module cache
	var __webpack_module_cache__ = {};
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
	// expose the modules object (__webpack_modules__)
	__webpack_require__.m = __webpack_modules__;
	// webpack/runtime/make_namespace_object
	!(function () {
		// define __esModule on exports
		__webpack_require__.r = function (exports) {
			if (typeof Symbol !== "undefined" && Symbol.toStringTag) {
				Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
			}
			Object.defineProperty(exports, "__esModule", { value: true });
		};
	})();
	// webpack/runtime/has_own_property
	!(function () {
		__webpack_require__.o = function (obj, prop) {
			return Object.prototype.hasOwnProperty.call(obj, prop);
		};
	})();
	// webpack/runtime/jsonp_chunk_loading
	!(function () {
		__webpack_require__.b = document.baseURI || self.location.href;

		// object to store loaded and loading chunks
		// undefined = chunk not loaded, null = chunk preloaded/prefetched
		// [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
		var installedChunks = { main: 0 };
	})();
	var __webpack_exports__ = __webpack_require__("./index.js");
})();
