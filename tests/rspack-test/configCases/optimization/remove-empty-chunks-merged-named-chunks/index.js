it("should load the chunks after the merged named chunk becomes empty", async () => {
	const first = await import(/* webpackChunkName: "first" */ "./x");
	const second = await import(/* webpackChunkName: "second" */ "./x");
	expect(second).toBe(first);
	expect(first.default).toBe("x");
});
