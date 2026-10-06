import value from "./file";

it("should write raw packs when compression is disabled", async () => {
	if (COMPILER_INDEX === 0 || COMPILER_INDEX === 1) {
		expect(value).toBe(1);
		await NEXT_START();
	} else {
		expect(value).toBe(2);
	}
});
