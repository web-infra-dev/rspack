it("preserves shared values in all async routes", async () => {
	const routes = await Promise.all([
		import(/* webpackChunkName: "same-route-a" */ "./route-a"),
		import(/* webpackChunkName: "same-route-b" */ "./route-b"),
		import(/* webpackChunkName: "same-route-c" */ "./route-c")
	]);
	expect(routes.map(route => route.default)).toEqual([
		[0, 1, 2, 3, 4, 5, 6, 7],
		[0, 1, 2, 3, 4, 5, 6, 7],
		[0, 1, 2, 3, 4, 5, 6, 7]
	]);
});
