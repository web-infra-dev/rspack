it("should execute a restored remote module", async () => {
	expect((await import("remote/value")).default).toBe("remote value");
	if (COMPILER_INDEX === 0) await NEXT_START();
});
