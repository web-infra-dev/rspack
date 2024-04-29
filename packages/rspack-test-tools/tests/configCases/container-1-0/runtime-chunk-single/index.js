it("should have single runtime chunk", () => {
	return import("./bootstrap").then(() => {
		const { entrypoints } = __STATS__;
		const entries = Object.keys(entrypoints);
		expect(entries).toContain("main");
		expect(entries).toContain("A");
		for (const entry of entries) {
			expect(entrypoints[entry].chunks).toContain("runtime")
		}
	})
});
