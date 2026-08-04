const asset = new URL("./asset.txt", import.meta.url);

it("should use the asset module value without a module dispatcher", () => {
	expect(asset.href).toMatch(/\.txt$/);
});
