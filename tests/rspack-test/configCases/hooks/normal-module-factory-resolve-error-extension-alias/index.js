const value = require("./missing.mjs");

it("should recover an extension-alias miss by retrying", () => {
	expect(value).toBe("extension-alias-fallback");
});
