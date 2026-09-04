it("should drop unused data properties with a side-effect-free value", () => {
	const m = require("./pure");
	expect(m.used).toBe("used");
	expect(m.getCount()).toBe(0);
});
