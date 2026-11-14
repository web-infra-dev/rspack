require("./direct");
require("./barrel");

it("should invalidate cached PURE requires when dependency side effects change", async () => {
	expect(globalThis.pureRequireCacheEffects || []).toEqual(
		COMPILER_INDEX === 1 ? ["direct", "transitive"] : []
	);
	delete globalThis.pureRequireCacheEffects;
	if (COMPILER_INDEX < 2) await NEXT_START();
});
