it("should explicitly restart without requiring file update separators", async () => {
	recordCompiler(COMPILER_INDEX);
	if (COMPILER_INDEX === 0) await NEXT_START();
	expect(COMPILER_INDEX).toBeLessThanOrEqual(1);
});
