it("should work", () => {
	const stats = __STATS__;
	expect(
		stats.modules.filter(m => m.name.startsWith("./esm")).length === 2
	).toBe(true);
});
