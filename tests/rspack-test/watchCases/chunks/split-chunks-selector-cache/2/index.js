it("reuses selectors only within this rebuild and preserves edited values", async () => {
	const routes = await Promise.all([
		import(/* webpackChunkName: "route-a" */ "./route-a"),
		import(/* webpackChunkName: "route-b" */ "./route-b"),
		import(/* webpackChunkName: "route-c" */ "./route-c")
	]);
	expect(routes.map(route => route.default)).toEqual([
		[100, 1, 2, 3, 4, 5, 6, 7],
		[100, 1, 2, 3, 4, 5, 6, 7],
		[4, 5, 6, 7]
	]);
});
