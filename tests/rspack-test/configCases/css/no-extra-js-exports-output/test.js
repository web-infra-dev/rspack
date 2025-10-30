it("should work", () => {
	const stats = __STATS__.children[__STATS_I__];

	expect(stats.assets.findIndex(a => a.name === "test.js") > -1).toBe(true);

	expect(
		stats.assets.findIndex(a => a.name === `${__STATS_I__}/main.css`) > -1
	).toBe(true);

	if (__STATS_I__ === 0) {
		// ./main.css
		// ./a.css
		// and it still output two runtime module:
		// 	 'webpack/runtime/make namespace object'
		// 	 'webpack/runtime/css loading'
		// DIFF: rspack generate extra js modules for css modules
		// expect(stats.modules.length).toBe(4);
		expect(stats.modules.length).toBe(8);
	} else if (__STATS_I__ === 1) {
		stats.modules
			.filter(module => module.moduleType === "css/auto")
			.forEach(module => {
				// DIFF: rspack generate js modules for size 42
				// expect(module.sizes["javascript"] === 1).toBe(true);
				expect(module.sizes["javascript"] === 42).toBe(true);
			});
	} else if (__STATS_I__ === 2) {
		stats.modules
			.filter(module => module.moduleType === "css/auto")
			.forEach(module => {
				expect(module.sizes["javascript"] === 1).toBe(false);
			});
	}
});
