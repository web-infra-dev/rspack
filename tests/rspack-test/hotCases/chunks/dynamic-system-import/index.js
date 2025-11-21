it("should import a changed chunk (dynamic import)", async () => {
	async function load(name) {
		return import("./chunk" + name);
	}
	const chunk = await load(1);
	expect(chunk.value).toBe(1);
	await NEXT_HMR();
	expect(chunk.value).toBe(2);
	const chunk2 = await load(2);
	expect(chunk2.value).toBe(2);
});

module.hot.accept(["./chunk1", "./chunk2"]);