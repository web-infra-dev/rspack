import value from "./file";

it("should invalidate the persistent cache when a file changes at the same mtime", async () => {
	if (COMPILER_INDEX < 2) {
		expect(value).toBe("before");
		const stats = await NEXT_START();
		const source = stats.modules.find(module => module.name === "./file.js");
		expect(source?.built).toBe(COMPILER_INDEX === 1);
	} else {
		expect(value).toBe("after!");
	}
});
