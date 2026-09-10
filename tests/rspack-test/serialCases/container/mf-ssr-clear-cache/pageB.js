import value from "remoteA/B";

globalThis.__mfSsrClearCacheRemoteServer.recordRouteExecution("pageB");

export function render() {
	return `pageB:${value}`;
}
