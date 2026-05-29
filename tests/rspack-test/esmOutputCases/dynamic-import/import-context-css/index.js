it("should support css loading in import.meta.glob target", async () => {
	const modules = import.meta.glob("./modules/*.js");
	const value = await modules["./modules/a.js"]();

	expect(value.default).toBe("a");
});
