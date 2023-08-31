const RefreshRuntime = require("react-refresh/runtime");

let _globalThis;
try {
	_globalThis = globalThis;
} catch(e) {
	_globalThis = window;
}
RefreshRuntime.injectIntoGlobalHook(_globalThis);
_globalThis.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;
