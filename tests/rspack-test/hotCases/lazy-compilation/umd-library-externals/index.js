it("first activation of a lazy import with a UMD external must not throw", async () => {
	let resolved;
	const promise = import("./module").then(r => (resolved = r));
	expect(resolved).toBe(undefined);
	await new Promise(resolve => setTimeout(resolve, 1000));
	await NEXT_HMR();
	const result = await promise;
	expect(result).toHaveProperty("default", "answer=42");
});
