import value from "./file";

it("should keep one compiler cache directory across versions", async () => {
	expect(value).toBe(COMPILER_INDEX + 1);

	if (COMPILER_INDEX < 2) {
		await NEXT_START();
	}
});
