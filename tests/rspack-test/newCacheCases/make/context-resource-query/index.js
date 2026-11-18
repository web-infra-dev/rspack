it("should keep context queries and fragments out of filesystem dependencies", async () => {
	const context = require.context("./modules", false, /a\.js$/);
	const result = context("./a.js");
	expect(result.query).toBe("?cats=meow");
	expect(result.resource).toMatch(/\?cats=meow#dogs$/);
	if (COMPILER_INDEX === 0) await NEXT_START();
});
