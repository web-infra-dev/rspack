import value, { changed } from "./test_lib";

it("should snapshot unmanaged-paths work", async () => {
	if (COMPILER_INDEX == 0) {
		expect(value).toBe(1);
		expect(changed).toBe(1);
		await NEXT_HMR();
		expect(value).toBe(2);
		expect(changed).toBe(2);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(value).toBe(2);
		expect(changed).toBe(3);
		await NEXT_HMR();
		expect(value).toBe(4);
		expect(changed).toBe(4);
	}
});
