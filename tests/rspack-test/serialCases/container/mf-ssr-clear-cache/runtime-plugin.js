const path = require("path");

function clearLocalRemoteRequireCache(remoteEntryPath) {
	const Module = __non_webpack_require__("node:module");
	const nativeRequireCache =
		__non_webpack_require__.cache || Module._cache || {};
	const remoteDir = path.dirname(remoteEntryPath);
	for (const cachePath of Object.keys(nativeRequireCache)) {
		const basename = path.basename(cachePath);
		if (
			path.dirname(cachePath) === remoteDir &&
			(basename === "remoteEntry.js" || basename.startsWith("remote-"))
		) {
			delete nativeRequireCache[cachePath];
			if (Module._cache && Module._cache !== nativeRequireCache) {
				delete Module._cache[cachePath];
			}
		}
	}
}

module.exports = function () {
	return {
		name: "mf-ssr-clear-cache-local-remote",
		loadEntry({ remoteInfo }) {
			if (remoteInfo.name !== "remoteA" && remoteInfo.alias !== "remoteA") {
				return;
			}
			const remoteEntryPath = path.resolve(__dirname, "remoteEntry.js");
			globalThis.__mfSsrClearCacheRemoteServer.clearProviderRuntime();
			clearLocalRemoteRequireCache(remoteEntryPath);
			const remoteEntry =
				__non_webpack_require__(remoteEntryPath)[remoteInfo.entryGlobalName];
			return globalThis.__mfSsrClearCacheRemoteServer.wrapRemoteEntry(
				remoteEntry,
				remoteInfo.entry
			);
		}
	};
};
