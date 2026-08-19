it("should preserve the resolution base for node-commonjs externals", async () => {
	const { relativeValue, packageValue } = await import("./lazy.cjs");

	expect(relativeValue).toBe(42);
	expect(packageValue).toBe("root");
});
