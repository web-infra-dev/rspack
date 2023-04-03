// Port from https://github.com/pmmmwh/react-refresh-webpack-plugin/blob/main/lib/runtime/RefreshUtils.js
// @ts-ignore
const RefreshUtils = require('@pmmmwh/react-refresh-webpack-plugin/lib/runtime/RefreshUtils');
const RefreshRuntime = require("react-refresh/runtime");

RefreshRuntime.injectIntoGlobalHook(globalThis);

function refresh(moduleId, webpackHot) {
	const currentExports = RefreshUtils.getModuleExports(moduleId);
	const fn = () => {
		RefreshUtils.executeRuntime(currentExports, moduleId, webpackHot);
	}
	if (typeof Promise !== 'undefined' && currentExports instanceof Promise) {
		currentExports.then(fn);
	} else {
		fn();
	}
}

// @ts-ignored
__webpack_modules__.$ReactRefreshRuntime$ = {
	refresh,
	register: RefreshRuntime.register,
	createSignatureFunctionForTransform: RefreshRuntime.createSignatureFunctionForTransform
};
