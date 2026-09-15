it("should preserve the require receiver for a CommonJS context", async () => {
	const name = "value";
	const value = await import(`./cjs/${name}.js`);

	expect(value.default).toBe("cjs");
});
