it("loads async chunks", async () => {
	const [{ value }, style] = await Promise.all([
		import("./lazy"),
		import("./style.css")
	]);

	expect(value).toBe(42);
	expect(style).toEqual(nsObj({}));
});
