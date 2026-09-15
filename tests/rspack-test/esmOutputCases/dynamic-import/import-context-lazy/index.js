it("should support lazy import context", async () => {
	const name = "a";
	const value = await import(`./modules/${name}.js`);

	expect(value.default).toBe("a");
});

it("should support lazy import.meta.glob", async () => {
	const modules = import.meta.glob("./modules/*.js");

	expect(Object.keys(modules).sort()).toEqual([
		"./modules/a.js",
		"./modules/b.js"
	]);

	const value = await modules["./modules/b.js"]();
	expect(value.default).toBe("b");
});
