import value from "remoteA/A";

globalThis.__mfSsrClearCacheRemoteServer.recordRouteExecution("pageA");

export function render() {
	return `pageA:${value}`;
}
