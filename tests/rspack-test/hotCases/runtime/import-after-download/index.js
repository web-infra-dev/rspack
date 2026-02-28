import value from "./file";

it("should download the missing update chunk on import", async () => {
	expect(value).toBe(1);
	const [chunk, unaffectedChunk] = await Promise.all([
		import("./chunk"),
		import("./unaffected-chunk")
	]);
	expect(value).toBe(1);
	expect(chunk.default).toBe(10);
	expect(unaffectedChunk.default).toBe(10);
	await NEXT_HMR();
	expect(value).toBe(2);
	expect(chunk.default).toBe(20);
	expect(unaffectedChunk.default).toBe(10);
});

module.hot.accept("./file");