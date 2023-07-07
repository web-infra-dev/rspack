// Thanks https://github.com/pmmmwh/react-refresh-webpack-plugin

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

globalThis.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;

__webpack_modules__.$ReactRefreshRuntime$ = {
	refresh,
	register: RefreshRuntime.register,
	createSignatureFunctionForTransform:
		RefreshRuntime.createSignatureFunctionForTransform
};
