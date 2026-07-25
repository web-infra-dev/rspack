const { DerivedA } = require("./module-a");
const { DerivedB } = require("./module-b");

it("should share TypeScript class inheritance fallbacks", () => {
	const derivedA = new DerivedA();
	expect(derivedA.value()).toBe("base-a");
	expect(DerivedA.staticValue).toBe("static-a");

	const derivedB = new DerivedB();
	expect(derivedB.value()).toBe("base-b");
	expect(DerivedB.staticValue).toBe("static-b");
});
