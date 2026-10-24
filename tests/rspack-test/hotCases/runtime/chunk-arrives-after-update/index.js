it("should keep the updated factory when a chunk requested before the update arrives after it", async () => {
	// Hold the script request so the chunk arrives after the update is applied.
	// Only script-based chunk loading goes through this function.
	const loadScript = __webpack_require__.l;
	let release;
	const released = new Promise((resolve) => {
		release = resolve;
	});
	__webpack_require__.l = function (...args) {
		released.then(() => loadScript.apply(this, args));
	};
	const pending = import("./chunk");
	__webpack_require__.l = loadScript;

	await NEXT_HMR();
	release();
	await pending;
	expect((await import("./chunk")).default).toBe(2);
});
