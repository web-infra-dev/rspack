const remoteVersions = ["v1", "v2", "v3", "v4"];
const exposedModuleIds = remoteVersions.flatMap(version => [
	`./remote-projects/${version}/remoteA.js`,
	`./remote-projects/${version}/remoteB.js`
]);
const exposedChunkIds = remoteVersions.flatMap(version => [
	`remote-projects_${version}_remoteA_js`,
	`remote-projects_${version}_remoteB_js`
]);

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
