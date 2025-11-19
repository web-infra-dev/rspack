it("should not respect .swcrc", () => {
	const { a } = require("./a.ts");
	expect(a).toBe(42);
});
