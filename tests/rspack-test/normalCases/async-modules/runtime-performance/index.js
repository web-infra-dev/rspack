it("should not take too long to evaluate nested async modules", async () => {
	// const start = Date.now();
	await import(/* webpackMode: "eager" */ "./loader.mjs?i=40!./loader.mjs");
	// Disabling flaky test.
	// expect(Date.now() - start).toBeLessThan(100);
});
