import value from "./file";

it("should invalidate persistent cache when loader changes", async () => {
	if (COMPILER_INDEX === 0) {
		expect(value).toBe(1);
		await NEXT_START();
	}
	if (COMPILER_INDEX === 1) {
		expect(value).toBe(2);
	}
});
