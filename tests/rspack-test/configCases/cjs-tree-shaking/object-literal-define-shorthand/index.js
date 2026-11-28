it("should keep a shorthand property whose identifier is replaced by another dependency", () => {
	const m = require("./lib");
	expect(m.used).toBe("used");
	expect(m.DEFINED_USED).toBe("defined-used");
	expect(m.usedExports).toEqual(["DEFINED_USED", "used", "usedExports"]);
});
