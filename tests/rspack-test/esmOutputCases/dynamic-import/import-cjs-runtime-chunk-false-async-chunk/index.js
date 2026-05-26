it("should split runtime for Promise.all with external and bundled async chunks", async () => {
	const [stream, mod] = await Promise.all([
		import("node:stream"),
		import("./dynamic"),
	]);

	expect(stream.Readable).toBeDefined();
	expect(mod.value).toBe(42);
});
