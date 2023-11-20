it("should handle sub properties of module exports correctly", () => {
	const mod = require("./module-exports.js");
	expect(mod.aaa()).toBe(1);
	expect(mod.aaa.bbb()).toBe(2);
	expect(mod.ccc()).toBe(3);
	expect(mod.ccc.ddd()).toBe(4);
});
it("should handle sub properties of this correctly", () => {
	const mod = require("./this.js");
	expect(mod.aaa()).toBe(1);
	expect(mod.aaa.bbb()).toBe(2);
	expect(mod.ccc()).toBe(3);
	expect(mod.ccc.ddd()).toBe(4);
});
it("should handle sub properties of exports correctly", () => {
	const mod = require("./exports.js");
	expect(mod.aaa()).toBe(1);
	expect(mod.aaa.bbb()).toBe(2);
	expect(mod.ccc()).toBe(3);
	expect(mod.ccc.ddd()).toBe(4);
});
