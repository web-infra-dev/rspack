it("should not count an excluded named destination towards minChunks", async () => {
	const modules = await Promise.all([
		import(/* webpackChunkName: "Target" */ "./util"),
		import(/* webpackChunkName: "Foo" */ "./Foo"),
		import(/* webpackChunkName: "Bar" */ "./Bar")
	]);

	expect(modules.map(module => module.default)).toEqual([
		"util",
		"Foo:util:prelude",
		"Bar:util"
	]);
});
