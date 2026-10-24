it("should restore the latest factory after an old chunk replaces a removed module", async () => {
	const moduleId = "./x.js";
	const loadScript = __webpack_require__.l;
	let release;
	const released = new Promise(resolve => {
		release = resolve;
	});

	// Capture the original chunk URL, but deliver it after the first update.
	__webpack_require__.l = function (...args) {
		released.then(() => loadScript.apply(this, args));
	};
	let pending;
	try {
		pending = import("./a");
	} finally {
		__webpack_require__.l = loadScript;
	}

	// Remove x from the build while the old a chunk is still in flight.
	await NEXT_HMR();
	const removedFactory = __webpack_require__.m[moduleId];
	expect(removedFactory).toBeDefined();
	expect(__webpack_require__.c[moduleId]).toBeUndefined();

	// The old chunk can replace the removed-module placeholder with x v1.
	release();
	await pending;
	const staleFactory = __webpack_require__.m[moduleId];
	expect(staleFactory).not.toBe(removedFactory);
	expect(__webpack_require__.c[moduleId]).toBeUndefined();

	// Restore x v2 only in b, whose chunk has never been requested.
	await NEXT_HMR();
	expect(__webpack_require__.m[moduleId]).toBe(staleFactory);
	expect(__webpack_require__.c[moduleId]).toBeUndefined();

	const latest = await import("./b");
	// x has not executed yet, so a stale result cannot come from its cache.
	expect(__webpack_require__.c[moduleId]).toBeUndefined();
	expect(latest.read()).toBe(2);
});
