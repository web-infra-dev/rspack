it("should keep a module still reached by an active require edge", () => {
	const m = require("./lib");
	expect(m.used).toBe("used");
	expect(m.usedExports).toEqual(["used", "usedExports"]);
	expect(require("./heavy")).toBe("heavy");
	expect(require.resolveWeak("./heavy")).not.toBe(null);
});
