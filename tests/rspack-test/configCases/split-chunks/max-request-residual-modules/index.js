it("should only move modules from chunks kept by max request limits", async () => {
	const modules = await Promise.all([
		import(/* webpackChunkName: "A1" */ "./A"),
		import(/* webpackChunkName: "A2" */ "./A?1"),
		import(/* webpackChunkName: "D" */ "./D"),
		import(/* webpackChunkName: "B1" */ "./B"),
		import(/* webpackChunkName: "B2" */ "./B?1"),
		import(/* webpackChunkName: "C" */ "./C")
	]);

	expect(modules.map(module => module.default)).toEqual([
		"A:alpha:prelude",
		"A:alpha:prelude",
		"D:alpha",
		"B:beta:g",
		"B:beta:g",
		"C:beta"
	]);
});
