if (typeof document === "undefined") {
	self.onmessage = () => self.postMessage("worker-v1");
} else {
	globalThis.NEW_URL_SCRIPT_VERSION = "script-v1";
}
---
if (typeof document === "undefined") {
	self.onmessage = () => self.postMessage("worker-v2");
} else {
	globalThis.NEW_URL_SCRIPT_VERSION = "script-v2";
}
