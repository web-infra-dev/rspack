import "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should still infer empty exports for strict method-shorthand factories", () => {
	expect(globalThis.modernFunctionCallerTargetLoaded).toBe(true);
	expect(findModule("reflection.js").providedExports).toEqual([]);
});
