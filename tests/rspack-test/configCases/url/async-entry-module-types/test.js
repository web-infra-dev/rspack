it("should render new URL targets as async entries by module source type", () => {
	const stats = __STATS__.children[__STATS_I__];
	const assets = stats.assets.map(asset => asset.name);

	expect(assets).toContain("test.js");
	expect(assets).toContain(`${__STATS_I__}/main.js`);

	switch (__STATS_I__) {
		case 0:
			expect(stats.assets.some(asset => asset.info.sourceFilename === "target.png")).toBe(true);
			expect(assets.filter(name => name.endsWith(".js")).sort()).toEqual([
				"0/main.js",
				"test.js"
			]);
			break;
		case 1:
			expect(assets.some(name => name.endsWith(".css"))).toBe(true);
			expect(assets.some(name => name.startsWith("1/url-") && name.endsWith(".js"))).toBe(false);
			break;
		case 2:
			expect(assets.some(name => name.endsWith(".wasm"))).toBe(true);
			expect(assets.some(name => name.startsWith("2/url-") && name.endsWith(".js"))).toBe(false);
			break;
		case 3:
			expect(assets.some(name => name.startsWith("3/url-") && name.endsWith(".js"))).toBe(true);
			break;
	}
});
