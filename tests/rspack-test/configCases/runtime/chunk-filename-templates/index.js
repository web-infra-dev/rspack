it("generates chunk filenames from ID and hash placeholders", async () => {
	const { default: value } = await import(
		/* webpackChunkName: "12345" */ "./lazy.js"
	);
	expect(value).toBe(42);
});
