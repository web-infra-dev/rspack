const value = require("m1");

it("should recover an alias-target miss by retrying", () => {
	expect(value).toBe("alias-fallback");
});
