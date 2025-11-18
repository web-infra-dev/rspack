it("should not respect to .swcrc", () => {
	const { a } = require("./a.ts");
	expect(a).toBe(42);
});
