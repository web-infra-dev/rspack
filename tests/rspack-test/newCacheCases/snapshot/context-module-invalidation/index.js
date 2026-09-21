it("should invalidate a cached context when members are added, changed or removed", async () => {
	const context = require.context("./context", true, /\.js$/, "lazy");
	const stable = require.context("./stable", false, /\.js$/);
	expect(stable.keys()).toEqual(["./value.js"]);
	expect(stable("./value.js")).toBe("stable");

	if (COMPILER_INDEX < 2) {
		expect(context.keys()).toEqual(["./a.js"]);
		expect(await context("./a.js")).toBe("original");
	} else if (COMPILER_INDEX < 4) {
		expect(context.keys().sort()).toEqual(["./a.js", "./nested/added.js"]);
		expect(await context("./a.js")).toBe(COMPILER_INDEX === 2 ? "original" : "changed");
		expect(await context("./nested/added.js")).toBe("added");
	} else {
		expect(context.keys()).toEqual(["./nested/added.js"]);
		expect(await context("./nested/added.js")).toBe("added");
		await expect(context("./a.js")).rejects.toThrow("Cannot find module");
	}

	if (COMPILER_INDEX < 5) await NEXT_START();
});
