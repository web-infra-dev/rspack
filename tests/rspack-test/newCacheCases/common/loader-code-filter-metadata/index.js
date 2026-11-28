import value from "./entry.js";

it("should retain code filter metadata with new-cache persistent cache", async () => {
	expect(value).toBe(COMPILER_INDEX === 2 ? "__VALUE__" : "filtered");
	if (COMPILER_INDEX === 1) await NEXT_START();
});
