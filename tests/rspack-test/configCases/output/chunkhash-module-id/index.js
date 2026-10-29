it("should load the async chunk", async () => {
	const { default: value } = await import(/* webpackChunkName: "async" */ "./async");
	expect(value).toBe(42);
});
