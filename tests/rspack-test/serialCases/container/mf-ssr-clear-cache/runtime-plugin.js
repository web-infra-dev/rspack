const path = require("path");

function getRemoteEntry(remoteInfo) {
	const entry =
		remoteInfo.entry ||
		remoteInfo.external ||
		remoteInfo.url ||
		remoteInfo.entryUrl;
	const normalizedEntry = Array.isArray(entry) ? entry[0] : entry;
	if (!normalizedEntry) {
		throw new Error(
			`remoteA is missing entry: ${JSON.stringify(remoteInfo, null, 2)}`
		);
	}
	const entryUrl = String(normalizedEntry).split("@").pop();
	const { pathname } = new URL(entryUrl, "http://localhost");
	return {
		entry: entryUrl,
		filename: decodeURIComponent(pathname).replace(/^\/+/, "")
	};
}

module.exports = function () {
	return {
		name: "mf-ssr-clear-cache-local-remote",
		loadEntry({ remoteInfo }) {
			if (remoteInfo.name !== "remoteA" && remoteInfo.alias !== "remoteA") {
				return;
			}
			const { entry, filename } = getRemoteEntry(remoteInfo);
			const remoteEntryPath = path.resolve(__dirname, filename);
			const remoteEntry =
				__non_webpack_require__(remoteEntryPath)[remoteInfo.entryGlobalName];
			return globalThis.__mfSsrClearCacheRemoteServer.wrapRemoteEntry(
				remoteEntry,
				entry
			);
		}
	};
};
