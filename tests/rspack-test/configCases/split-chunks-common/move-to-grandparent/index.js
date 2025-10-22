it("should correctly include indirect children in common chunk", async function() {
	await Promise.all([
		import('./pageA'),
		import('./pageB')
	]).then((imports) => {
		expect(imports[0].default).toBe("reuse");
		expect(imports[1].default).toBe("reuse");
	})
});
