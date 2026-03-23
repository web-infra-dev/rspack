it("should merge duplicate externals inside a dynamic multi-module chunk", async () => {
	const modA = await import(/* webpackChunkName: "shared" */ "./a.js");
	const modB = await import(/* webpackChunkName: "shared" */ "./b.js");

	expect(modA.fsDefault).toBeDefined();
	expect(modA.readFileSync).toBeDefined();
	expect(modB.readFile).toBe(modA.readFile);
	expect(modB.marker).toBe("b");
});
