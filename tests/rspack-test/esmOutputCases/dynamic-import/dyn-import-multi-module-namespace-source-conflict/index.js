it("should avoid namespace binding collisions across different external sources", async () => {
	const modA = await import(/* webpackChunkName: "shared" */ "./a.js");
	const modB = await import(/* webpackChunkName: "shared" */ "./b.js");

	expect(modA.fsNs).toBeDefined();
	expect(modA.fsNs.readFile).toBeDefined();
	expect(modB.pathNs).toBeDefined();
	expect(modB.pathNs.resolve).toBeDefined();
});
