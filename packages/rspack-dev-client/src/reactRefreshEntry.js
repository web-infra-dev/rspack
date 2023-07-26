const RefreshRuntime = require("react-refresh/runtime");

RefreshRuntime.injectIntoGlobalHook(globalThis);
globalThis.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;
