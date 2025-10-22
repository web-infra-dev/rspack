it("should regenerate contenthash", async () => {
	const value1 = new URL("./file.text", import.meta.url);
	expect(/file\.[\da-f]{16}\.text/.test(value1.toString())).toBe(true);
	await NEXT_HMR();
	const value2 = new URL("./file.text", import.meta.url);
	expect(/file\.[\da-f]{16}\.text/.test(value2.toString())).toBe(true);
	expect(value1.toString()).not.toBe(value2.toString());
});

module.hot.accept("./file.text");
