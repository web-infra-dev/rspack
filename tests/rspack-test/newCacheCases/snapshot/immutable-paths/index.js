import value from "./file";

it("should snapshot immutable-paths work", async () => {
	if (COMPILER_INDEX == 0) {
		expect(value).toBe(1);
		await NEXT_HMR();
		expect(value).toBe(2);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(value).toBe(2);
		await NEXT_HMR();
		expect(value).toBe(4);
	}
});

module.hot.accept("./file");