import value from "./value";

it("should preserve child cache statistics until the next build leaves idle", async () => {
	if (COMPILER_INDEX === 0) {
		expect(value).toBe(1);
		await NEXT_HMR();
	}
	expect(value).toBe(2);
});

module.hot.accept("./value");
