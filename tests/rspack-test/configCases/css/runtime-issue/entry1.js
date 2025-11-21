const img = new URL("./img.png", import.meta.url);

it("should allow to create css modules", async () => {
	const { default: x } = await import("./asyncChunk");
	expect(img.toString()).toBe("https://test.cases/path/img.png");
	expect(x.default.class).toEqual("_test_module_css-class");
});
