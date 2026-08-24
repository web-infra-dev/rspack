import value from "./file";

it("should keep the legacy make cache when the module new cache is disabled", async () => {
	expect(value).toBe(1);
	if (COMPILER_INDEX < 1) await NEXT_START();
});
