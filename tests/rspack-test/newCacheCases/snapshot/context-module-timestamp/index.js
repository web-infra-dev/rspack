it("should use the context snapshot strategy independently of the normal module strategy", async () => {
	const context = require.context("./context", false, /\.js$/);
	expect(context.keys()).toEqual(["./value.js"]);
	expect(context("./value.js")).toBe("value");
	if (COMPILER_INDEX < 3) await NEXT_START();
});
