it("should compile and run", async () => {
	expect(true).toBe(true);
	const module = await import("./async");
	expect(module.default).toBe("async");
});
