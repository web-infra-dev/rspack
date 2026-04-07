import a from "./a";

it("should recover after error with no-emit-on-errors", async () => {
	expect(a).toBe(1);

	// Step 1: syntax error - emitOnErrors: false prevents emit and record
	try {
		await NEXT_HMR();
	} catch (e) {
		// Expected: no update available because emit was prevented
	}
	expect(a).toBe(1);

	// Step 2: fix - HMR should compare against step 0 (last good compilation)
	await NEXT_HMR();
	expect(a).toBe(3);
});

module.hot.accept("./a");
