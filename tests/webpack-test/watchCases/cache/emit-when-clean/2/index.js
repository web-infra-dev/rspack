it("should not emit files", () => {
	expect(__STATS__.assets.map(a => a.name)).not.toContainEqual(
		expect.stringMatching(/\.txt$/)
	);
});
