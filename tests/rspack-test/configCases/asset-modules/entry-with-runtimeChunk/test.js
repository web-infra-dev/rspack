it("should work", () => {
	const stats = __STATS__.children[__STATS_I__];

	const test = stats.assets.find(
		a => a.name === "test.js"
	);
	expect(Boolean(test)).toBe(true);

	const assetEntry = stats.assets.find(
		a => a.info.sourceFilename === "../_images/file.png"
	);
	expect(Boolean(assetEntry)).toBe(true);

	if (__STATS_I__ % 4 === 0 || __STATS_I__ % 4 === 2) {
		expect(
			stats.assets.filter(a => /\.m?js$/.test(a.name)).map(a => a.name)
		).toEqual(["test.js"]);
	}
});
