it("should import a changed chunk", async () => {
	const chunk = await import("./chunk");
	expect(chunk.value).toBe(1);
	expect(chunk.value2).toBe(3);
	expect(chunk.counter).toBe(0);
	await NEXT_HMR();
	expect(chunk.value).toBe(2);
	expect(chunk.value2).toBe(4);
	expect(chunk.counter).toBe(1);
	const chunk2 = await import("./chunk2");
	expect(chunk2.value).toBe(2);
	expect(chunk2.value2).toBe(4);
	expect(chunk2.counter).toBe(0);
});
