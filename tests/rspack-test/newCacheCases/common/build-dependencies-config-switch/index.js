it("should invalidate changes to configured build dependencies", async () => {
	if (COMPILER_INDEX < 7) await NEXT_START();
});
