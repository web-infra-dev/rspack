it("should not replace defined identifier", function () {
	expect(() => require("./mod")).not.toThrow();
	const { foo, Mod } = require("./mod");
	expect(foo).toBe(Mod);
});
