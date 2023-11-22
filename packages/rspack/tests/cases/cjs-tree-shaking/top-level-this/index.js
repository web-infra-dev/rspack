it("should handle top level this in the root scope", () => {
	const mod = require("./root.js");
	expect(mod.aaa).toBe(1);
	expect(mod.bbb).toBe(2);
});
it("should handle top level this in block scope", () => {
	const mod = require("./block.js");
	expect(mod.aaa).toBe(1);
	expect(mod.bbb).toBe(2);
});
