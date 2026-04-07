import "./logo.png";

it("should work with image-minimizer-webpack-plugin in plugin mode", () => {
	const asset = __STATS__.assets.find(item => item.name.endsWith(".png"));
	expect(asset).toBeDefined();
});
