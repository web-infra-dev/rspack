it("should convert TypeScript to JavaScript", () => {
	const { Foo } = require("./lib");
	expect(new Foo().foo()).toBe(42);
});

it("should have swc/helpers", () => {
	expect(
		Object.keys(__webpack_modules__).some(name => name.includes("swc/helpers"))
	).toBe(true);
});
