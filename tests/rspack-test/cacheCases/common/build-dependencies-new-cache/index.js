import value from "./file";

it("should validate new cache build dependencies", async () => {
	expect(value).toBe(1);
	if (COMPILER_INDEX < 5) await NEXT_START();
});
