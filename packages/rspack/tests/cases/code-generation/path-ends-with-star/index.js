it("should generate valid code", async () => {
	if (process.platform !== "win32") {
		const { staticA, dynamicA } = await import("./entry.mjs");
		expect(staticA.a).toBe(1);
		expect(dynamicA.a).toBe(1);
	} else {
		expect("skip windows").toBe("skip windows");
	}
});
