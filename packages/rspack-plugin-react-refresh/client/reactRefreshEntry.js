/**
 * The following code is modified based on
 * https://github.com/pmmmwh/react-refresh-webpack-plugin/blob/0b960573797bf38926937994c481e4fec9ed8aa6/client/ReactRefreshEntry.js
 *
 * MIT Licensed
 * Author Michael Mok
 * Copyright (c) 2019 Michael Mok
 * https://github.com/pmmmwh/react-refresh-webpack-plugin/blob/0b960573797bf38926937994c481e4fec9ed8aa6/LICENSE
 */

var RefreshRuntime = require("react-refresh/runtime");
var safeThis = (function () {
	// copied from core-js-pure/features/global-this
	"use strict";

	var check = function (it) {
		return it && it.Math == Math && it;
	};

	// https://github.com/zloirock/core-js/issues/86#issuecomment-115759028
	// eslint-disable-next-line es/no-global-this -- safe
	return (
		check(typeof globalThis == "object" && globalThis) ||
		check(typeof window == "object" && window) ||
		// eslint-disable-next-line no-restricted-globals -- safe
		check(typeof self == "object" && self) ||
		check(typeof __webpack_require__.g == "object" && __webpack_require__.g) ||
		// eslint-disable-next-line no-new-func -- fallback
		(function () {
			return this;
		})() ||
		this ||
		Function("return this")()
	);
})();

if (process.env.NODE_ENV !== "production") {
	if (typeof safeThis !== "undefined") {
		var $RefreshInjected$ = "__reactRefreshInjected";
		// Namespace the injected flag (if necessary) for monorepo compatibility
		if (
			typeof __react_refresh_library__ !== "undefined" &&
			__react_refresh_library__
		) {
			$RefreshInjected$ += "_" + __react_refresh_library__;
		}

		// Only inject the runtime if it hasn't been injected
		if (!safeThis[$RefreshInjected$]) {
			RefreshRuntime.injectIntoGlobalHook(safeThis);

			// Empty implementation to avoid "ReferenceError: variable is not defined" in module which didn't pass builtin:react-refresh-loader
			safeThis.$RefreshSig$ = () => type => type;
			safeThis.$RefreshReg$ = () => {};

			// Mark the runtime as injected to prevent double-injection
			safeThis[$RefreshInjected$] = true;
		}
	}
}
