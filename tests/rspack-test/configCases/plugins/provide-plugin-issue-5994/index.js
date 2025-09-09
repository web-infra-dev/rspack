it("should not replace defined identifier", function () {
	expect(() => require("./mod")).not.toThrow();
	const { foo, Mod } = require("./mod");
	expect(foo).toBe(Mod);
});

it("should not replace defined identifier that is exported default", function () {
	expect(() => require("./default")).not.toThrow();
	const { foo, default: def } = require("./default");
	expect(foo).toBe(def);
});
