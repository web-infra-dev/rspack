// @ts-ignore
const RefreshRuntime = require("react-refresh/runtime");

function debounce(fn: Function, delay: number) {
	var handle: number | undefined;
	return () => {
		clearTimeout(handle);
		handle = setTimeout(fn, delay);
	};
}

RefreshRuntime.injectIntoGlobalHook(globalThis);
globalThis.$RefreshReg$ = () => {};
globalThis.$RefreshSig$ = () => type => type;

var queueUpdate = debounce(RefreshRuntime.performReactRefresh, 16);

// @ts-ignored
__webpack_modules__.$ReactRefreshRuntime$ = {
	queueUpdate,
	...RefreshRuntime
};
