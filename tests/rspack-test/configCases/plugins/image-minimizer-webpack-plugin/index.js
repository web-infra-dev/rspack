import "./logo.png";

it("should minimize images with plugin mode", () => {
	const asset = __STATS__.assets.find(item => item.name.endsWith(".png"));
	expect(asset).toBeDefined();
	expect(asset.info.minimized).toBe(true);
});
