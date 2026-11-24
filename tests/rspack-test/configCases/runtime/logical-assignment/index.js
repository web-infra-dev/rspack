if (globalThis.__neverLoaded) {
	import(/* webpackPrefetch: true */ "./lazy");
}

it("should create a fake namespace object for CommonJS dynamic import", async () => {
	const namespace = await import("./commonjs");
	expect(namespace.default).toBe("commonjs");
});
