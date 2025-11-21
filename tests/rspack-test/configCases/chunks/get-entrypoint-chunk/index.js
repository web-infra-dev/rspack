it("should compile", async () => {
	const { a } = await import(/* webpackChunkName: "async" */ "./async");
	expect(a).toBe(1);
});
