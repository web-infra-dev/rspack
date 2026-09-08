import value from "alias_file";

it("should snapshot missing dependencies work", async () => {
	if (COMPILER_INDEX == 0) {
		expect(value).toBe("file2");
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(value).toBe("file1");
	}
});

module.hot.accept("alias_file");
