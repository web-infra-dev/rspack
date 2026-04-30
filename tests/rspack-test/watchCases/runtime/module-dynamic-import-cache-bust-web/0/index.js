it("should load dynamic chunk in development web module mode", async () => {
	const step = WATCH_STEP;
	const mod = await import(/* webpackChunkName: "dynamic" */ "./dynamic");
	if (step === "0") {
		expect(mod.default).toBe("Normal");
	} else {
		expect(mod.default).toBe("Changed");
	}
});
