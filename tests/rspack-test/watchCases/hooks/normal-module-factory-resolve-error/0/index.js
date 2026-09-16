it("should rebuild with the real module once the missing file appears", () => {
	const value = require("./target");
	if (WATCH_STEP === "0") {
		expect(value).toBe("fallback");
	} else {
		expect(value).toBe("target");
	}
});
