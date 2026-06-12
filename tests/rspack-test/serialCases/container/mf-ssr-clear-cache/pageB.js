import value from "remoteA/B";

globalThis.__mfSsrClearCacheHarness.recordRouteExecution("pageB");

export function render() {
	return `pageB:${value}`;
}
