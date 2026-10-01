import value from "./file";

it("should compress packs and preserve output across cold restarts", async () => {
	if (COMPILER_INDEX === 0 || COMPILER_INDEX === 1) {
		expect(value).toBe(1);
		await NEXT_START();
	} else {
		expect(value).toBe(2);
	}
});
