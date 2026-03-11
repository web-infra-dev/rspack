it("should respect strictThisContextOnImports for member call", () => {
	let m = require("./dir4/a");
	expect(m.f()).toBe(1);
	expect(m.usedExports).toBe(true);
	let m2 = require("./dir4/lib");
	expect(m2.b.f()).toBe(1);
	expect(m2.b.usedExports).toBe(true);
	expect(m2.usedExports).toEqual(["b", "usedExports"]);
})
