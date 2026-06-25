it("should wrap multi chunk context ensure calls", async () => {
	const modules = import.meta.glob("./modules/*.js");
	const value = await modules["./modules/a.js"]();
	const name = "b";
	const dynamicValue = await import(`./modules/${name}.js`);

	expect(value.default).toBe(42);
	expect(dynamicValue.default).toBe(43);
	expect(Object.keys(modules).sort()).toEqual([
		"./modules/a.js",
		"./modules/b.js",
	]);
});
