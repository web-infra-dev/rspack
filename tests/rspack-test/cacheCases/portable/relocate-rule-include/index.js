import value from "./src/dep";

it("should stop applying a rule whose fixed include no longer matches after relocation", async () => {
	// The rule's include names the original checkout. It matches there and nowhere else, just
	// like a cold build at the new location.
	expect(value).toBe(COMPILER_INDEX < 2 ? "transformed" : "original");
	if (COMPILER_INDEX === 0) {
		const stats = await NEXT_START(false);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
	if (COMPILER_INDEX === 1) {
		const stats = await NEXT_MOVE_DIR_START();
		expect(stats.modules.find(module => module.name.endsWith("/dep.js")).built).toBe(true);
	}
	if (COMPILER_INDEX === 2) {
		const stats = await NEXT_START(false);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
});
