it("should load the latest factory of a removed module after an old chunk arrived during an update", async () => {
	const moduleId = "./x.js";
	const loadScript = __webpack_require__.l;
	let release;

	// Capture the original chunk URL, but deliver it while the first update is prepared.
	__webpack_require__.l = function (...args) {
		release = () => loadScript.apply(this, args);
	};
	let pending;
	try {
		pending = import("./a");
	} finally {
		__webpack_require__.l = loadScript;
	}
	module.hot.addStatusHandler(status => {
		if (status !== "prepare" || !release) return;
		setTimeout(release, 100);
		release = undefined;
	});

	// Remove x from the build while the old a chunk is still in flight.
	await NEXT_HMR();
	await pending;
	expect(__webpack_require__.c[moduleId]).toBeUndefined();

	// Restore x v2 only in b, whose chunk has never been requested.
	await NEXT_HMR();
	const latest = await import("./b");
	expect(latest.read()).toBe(2);
});
