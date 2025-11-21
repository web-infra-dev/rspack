it("should compile", () => {
	const value = require(process.env.NODE_ENV !== "production" ? "./a" : "./b");
	expect(value).toBe(42);
});
