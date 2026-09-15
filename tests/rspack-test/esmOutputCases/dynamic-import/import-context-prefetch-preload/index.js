it("should keep prefetch and preload handlers for context chunks", async () => {
	const modules = import.meta.glob("./modules/*.js");
	const prefetchName = "a";
	const preloadName = "b";
	const prefetchValue = await import(
		/* webpackPrefetch: true */ `./modules/${prefetchName}.js`
	);
	const preloadValue = await import(
		/* webpackPreload: true */ `./modules/${preloadName}.js`
	);

	expect(Object.keys(modules).sort()).toEqual([
		"./modules/a.js",
		"./modules/b.js",
		"./modules/c.js",
	]);
	expect(prefetchValue.default).toBe("a");
	expect(preloadValue.default).toBe("b");
});
