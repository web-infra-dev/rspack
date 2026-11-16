it("should share util when loading consumers of a reused destination", async () => {
	const foo = await import(/* webpackChunkName: "Foo" */ "./Foo");
	const bar = await import(/* webpackChunkName: "Bar" */ "./Bar");
	const reusable = await import(/* webpackChunkName: "ReusableUtil" */ "./util");

	expect([foo.default, bar.default, reusable.default]).toEqual([
		"Foo:util:prelude",
		"Bar:util:prelude",
		"util"
	]);
});
