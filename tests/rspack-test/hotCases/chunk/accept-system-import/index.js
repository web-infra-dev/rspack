

it("should import a changed chunk", async () => {
	let chunk = await import("./chunk");
	expect(chunk.value).toBe(1);
	let chunk2 = await import("./chunk2");
	expect(chunk2.value).toBe(1);
	await NEXT_HMR();
	chunk = await import("./chunk");
	expect(chunk.value).toBe(2);
	chunk2 = await import("./chunk2");
	expect(chunk2.value).toBe(2);
});

module.hot.accept(["./chunk", "./chunk2"]);