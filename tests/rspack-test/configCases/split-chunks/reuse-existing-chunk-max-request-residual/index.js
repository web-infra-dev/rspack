it("should preserve a reused destination when all other sources are pruned", async () => {
	const modules = await Promise.all([
		import(/* webpackChunkName: "Foo" */ "./Foo"),
		import(/* webpackChunkName: "ReusableUtil" */ "./util")
	]);

	expect(modules.map(module => module.default)).toEqual([
		"Foo:util:prelude",
		"util"
	]);
});
