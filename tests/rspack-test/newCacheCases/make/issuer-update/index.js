import content from "./file";

it("should issuer update", async () => {
	if (COMPILER_INDEX === 0) {
		expect(content).toBe("a");
		await NEXT_HMR();
		expect(content).toBe("b");
		await NEXT_START();
	}
	if (COMPILER_INDEX === 1) {
		expect(content).toBe("b");
	}
});

module.hot.accept("./file");