import value from "./loader.mjs!./immutable/file";

it("should snapshot context dependencies work", async () => {
	if (COMPILER_INDEX === 0) {
		expect(value).toBe(1);
		await NEXT_START();
	}
	if (COMPILER_INDEX === 1) {
		expect(value).toBe(2);
		await NEXT_START();
	}
	if (COMPILER_INDEX === 2) {
		expect(value).toBe(2);
	}
});

module.hot.accept("./immutable/file");
