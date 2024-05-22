it("should have valid export", async () => {
	const exports = await import('./lib')
	expect(exports).toContain('hello')
});
