const exposedModuleIds = ["./remoteA.js", "./remoteB.js"];
const exposedChunkIds = ["remoteA_js", "remoteB_js"];

module.exports = function () {
	globalThis.__mfSsrClearCacheRemoteServer.setProviderRuntime({
		clear() {
			for (const moduleId of exposedModuleIds) {
				delete __webpack_require__.c[moduleId];
			}
			for (const control of Object.values(
				__webpack_require__.chunkCacheControls || {}
			)) {
				if (typeof control.clear === "function") {
					control.clear(exposedChunkIds);
				}
			}
		}
	});

	return {
		name: "mf-ssr-clear-cache-provider-runtime"
	};
};
