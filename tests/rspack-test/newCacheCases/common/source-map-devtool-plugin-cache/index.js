import value from "./file";

it("should hit source map persistent cache on cold restart", async () => {
	const asyncMod = await import("./async-module");
	expect(asyncMod.value).toBe(42);

	if (COMPILER_INDEX == 0) {
		expect(value).toBe(1);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(value).toBe(1);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 2) {
		expect(value).toBe(2);
	}
});
