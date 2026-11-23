it("should use the virtual module written in thisCompilation on restart", async () => {
	const value = require("./virtual.js");
	if (COMPILER_INDEX === 0) {
		expect(value).toBe("first");
		await NEXT_START();
	} else {
		expect(value).toBe("second");
	}
});
