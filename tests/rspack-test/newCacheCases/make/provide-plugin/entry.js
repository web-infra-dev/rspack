import value from "./file";

it("should provide plugin works", async () => {
	if (COMPILER_INDEX === 0) {
		expect(value).toBe(1);
		expect(Mod).toBe("a");
		await NEXT_START();
	}
	if (COMPILER_INDEX === 1) {
		expect(value).toBe(2);
		expect(Mod).toBe("b");
		await NEXT_START();
	}
	if (COMPILER_INDEX === 2) {
		expect(value).toBe(3);
		await NEXT_START();
	}
	if (COMPILER_INDEX === 3) {
		expect(value).toBe(4);
	}
});
