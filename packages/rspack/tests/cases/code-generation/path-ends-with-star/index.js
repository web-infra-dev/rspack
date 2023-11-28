it("should generate valid code", async () => {
	if (process.platform !== "win32") {
		const { staticA, dynamicA } = await import("./entry.mjs");
		expect(staticA).toBe(1);
		expect(dynamicA).toBe(1);
	}
	expect("skip windows").toBe("skip windows");
});
