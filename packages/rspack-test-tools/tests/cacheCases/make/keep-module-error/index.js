import value from "./loader.js!./file";

it("should module error exist", async () => {
	if (COMPILER_INDEX === 0) {
		expect(value).toBe(1);
		await NEXT_START();
	}
	if (COMPILER_INDEX === 1) {
		expect(value).toBe(1);
	}
});
