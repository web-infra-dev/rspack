it("should restore eager and asynchronous shared fallback dependencies", async () => {
	expect((await import("shared-async")).default).toBe("shared fallback");
	expect((await import("shared-eager")).default).toBe("shared fallback");
	if (COMPILER_INDEX === 0) await NEXT_START();
});
