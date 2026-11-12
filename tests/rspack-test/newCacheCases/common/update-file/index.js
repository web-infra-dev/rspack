import value from "./file";

it("should basic test work", async () => {
	if (COMPILER_INDEX == 0) {
		expect(value).toBe(1);
		await NEXT_HMR();
		expect(value).toBe(2);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(value).toBe(2);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 2) {
		expect(value).toBe(2);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 3) {
		expect(value).toBe(3);
	}
});

module.hot.accept("./file");