import value from "./file";

it("should build dependencies work", async () => {
	if (COMPILER_INDEX == 0) {
		expect(value).toBe(1);
		await NEXT_HMR();
		expect(value).toBe(2);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(value).toBe(3);
		await NEXT_HMR();
		expect(value).toBe(4);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 2) {
		expect(value).toBe(5);
		await NEXT_HMR();
		expect(value).toBe(6);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 3) {
		expect(value).toBe(7);
		await NEXT_HMR();
		expect(value).toBe(8);
	}
});
