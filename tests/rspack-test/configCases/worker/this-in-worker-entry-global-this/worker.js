// a classic worker script's top-level `this` is its global scope
this.dependency = require("./dependency");

Object.defineProperty(this, "marker", { value: "defined" });
this.onmessage = event => {
	this.postMessage(
		`got ${event.data}:${this.marker}:${this === globalThis ? "globalThis" : "other"}:${this.dependency.value}:${this.dependency.thisIsExports ? "exports" : "other"}`
	);
};
