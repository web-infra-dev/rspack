it("should re-execute the entrypoint on update", async () => {
	let stats = await NEXT_HMR();
	expect(stats.errors.length).toBe(1);
	stats = await NEXT_HMR();
	expect(stats.errors.length).toBe(0);
	expect(global.STATE).toBe(1);
	delete global.STATE;
	try {
		await NEXT_HMR();
		throw new Error("should not be reached");
	} catch (e) {
		expect(e.message).toBe("Aborted because ./index.js is not accepted\nUpdate propagation: ./index.js");
		expect(e.stats.errors.length).toBe(0);
		return;
	}
});

// ignore errors
module.hot.accept(() => { });
---
)))
---
	global.STATE = 1;
throw new Error("root-error");
---
// will never happen
// but should lead to apply failing because of unaccepted module
