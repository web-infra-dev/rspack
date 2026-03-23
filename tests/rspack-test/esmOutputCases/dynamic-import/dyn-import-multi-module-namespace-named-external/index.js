it("should avoid namespace and named binding collisions for duplicate externals", async () => {
	const modA = await import(/* webpackChunkName: "shared" */ "./a.js");
	const modB = await import(/* webpackChunkName: "shared" */ "./b.js");

	expect(modA.resolve).toBeDefined();
	expect(typeof modA.resolve.join).toBe("function");
	expect(modA.resolve.resolve).toBe(modB.resolve);
	expect(modB.marker).toBe("b");
});
