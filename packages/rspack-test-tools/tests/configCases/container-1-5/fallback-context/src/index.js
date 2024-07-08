it("should work with different context in fallback module", async () => {
	const shared = await import("./shared");
	expect(shared.ok).toBe(true);
});
