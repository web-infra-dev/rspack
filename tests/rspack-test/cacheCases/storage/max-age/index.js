import value from "./file";

it("should expire persistent cache by age", async () => {
	expect(value).toBe(COMPILER_INDEX + 1);

	if (COMPILER_INDEX < 2) {
		await NEXT_START();
	}
});

