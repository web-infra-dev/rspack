import value from "./file";

it("should restore built modules from the new cache", async () => {
	expect(value).toBe(1);
	if (COMPILER_INDEX < 1) await NEXT_START();
});
