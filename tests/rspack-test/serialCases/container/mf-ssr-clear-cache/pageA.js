import value from "remoteA/A";

globalThis.__mfSsrClearCacheHarness.recordRouteExecution("pageA");

export function render() {
	return `pageA:${value}`;
}
