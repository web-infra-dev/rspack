import value from "./loader.mjs!./file";

it("should keep a missing context dependency valid on restart", async () => {
	expect(value).toBe(1);
	if (COMPILER_INDEX === 0) {
		const stats = await NEXT_START();
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
});
