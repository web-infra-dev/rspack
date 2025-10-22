import { value } from "./test";

it("should work", async () => {
	if (COMPILER_INDEX == 0) {
		expect(value).toBe('a');
		await NEXT_HMR();
		expect(value).toBe('b');
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(value).toBe('a');
		await NEXT_HMR();
		expect(value).toBe('b');
	}
});

module.hot.accept("./test");