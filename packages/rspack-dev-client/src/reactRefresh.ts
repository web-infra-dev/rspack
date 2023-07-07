// Thanks https://github.com/pmmmwh/react-refresh-webpack-plugin
// @ts-ignore
const RefreshUtils = require("@pmmmwh/react-refresh-webpack-plugin/lib/runtime/RefreshUtils");
const RefreshRuntime = require("react-refresh/runtime");

RefreshRuntime.injectIntoGlobalHook(globalThis);

// Port from https://github.com/pmmmwh/react-refresh-webpack-plugin/blob/main/loader/utils/getRefreshModuleRuntime.js#L29
function refresh(moduleId, webpackHot) {
	const currentExports = RefreshUtils.getModuleExports(moduleId);
	const fn = exports => {
		RefreshUtils.executeRuntime(exports, moduleId, webpackHot);
	};
	if (typeof Promise !== "undefined" && currentExports instanceof Promise) {
		currentExports.then(fn);
	} else {
		fn(currentExports);
	}
}

// Injected global react refresh runtime

// @ts-ignored
globalThis.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;

// @ts-ignored
module.exports = {
	refresh,
	register: RefreshRuntime.register,
	createSignatureFunctionForTransform:
		RefreshRuntime.createSignatureFunctionForTransform
};
