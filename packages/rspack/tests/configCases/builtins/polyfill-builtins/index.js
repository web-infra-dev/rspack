/**
 * It's hard to test polyfill-builtins because we need to set target to node to run
 */
it("url", () => {
	const util = require("util");
	expect(util.isNumber(1)).toBe(true);
	expect(util.isNumber("1")).toBe(false);
	expect(util.isString("1")).toBe(true);
	expect(util.isString(1)).toBe(false);
});
