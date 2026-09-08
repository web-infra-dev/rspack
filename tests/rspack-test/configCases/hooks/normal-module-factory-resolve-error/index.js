const missing = require("./missing-runtime");

it("should recover a failed normal module resolution by retrying", () => {
	expect(typeof missing).toBe("function");
	expect(() => missing()).toThrow("Cannot find module './missing-runtime'");
});
