import value from "./file";

it("should limit persistent cache versions", async () => {
	expect(value).toBe(COMPILER_INDEX + 1);

	if (COMPILER_INDEX < 3) {
		await NEXT_START();
	}
});

