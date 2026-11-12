it("should resolve import() to the latest evaluation of a self-accepting lazy module", async () => {
	const promise = import("./module");
	await NEXT_HMR_DEFERRED();
	expect(await promise).toHaveProperty("default", 1);
	await NEXT_HMR();
	expect(await import("./module")).toHaveProperty("default", 2);
	await NEXT_HMR();
	expect(await import("./module")).toHaveProperty("default", 3);
});
