it("should build a stylesheet entry that imports a css module", () => {
	const assets = __STATS__.assets.map((asset) => asset.name);
	expect(assets).toContain("main.css");
});
