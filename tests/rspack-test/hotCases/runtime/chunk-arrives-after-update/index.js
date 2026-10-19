it("should keep the updated factory when a chunk requested before the update arrives during it", async () => {
	// Hold the script request so the chunk arrives while the update is prepared.
	// Only script-based chunk loading goes through this function.
	const loadScript = __webpack_require__.l;
	let release;
	__webpack_require__.l = function (...args) {
		release = () => loadScript.apply(this, args);
	};
	const pending = import("./chunk");
	__webpack_require__.l = loadScript;
	module.hot.addStatusHandler(status => {
		if (status !== "prepare" || !release) return;
		// Late enough for an update that does not wait for the chunk to apply first.
		setTimeout(release, 100);
		release = undefined;
	});

	await NEXT_HMR();
	await pending;
	expect((await import("./chunk")).default).toBe(2);
});
