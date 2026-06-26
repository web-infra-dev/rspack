export function createRemoteExport(expose, version) {
	return globalThis.__mfSsrClearCacheRemoteServer.createLargeRemoteExport(
		expose,
		version
	);
}
