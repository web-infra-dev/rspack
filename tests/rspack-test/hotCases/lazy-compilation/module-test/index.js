it("should not lazily compile configured imports", async () => {
	let resolvedA;
	let resolvedB;
	const promiseA = import("./moduleA").then(r => (resolvedA = r));
	const promiseB = import("./moduleB").then(r => (resolvedB = r));
	expect(resolvedA).toBe(undefined);
	expect(resolvedB).toBe(undefined);

	await new Promise(resolve => setTimeout(resolve, 1000));

	expect(resolvedA).toBe(undefined);
	expect(resolvedB).toHaveProperty("default", "B");

	await NEXT_HMR();
	const result = await promiseA;
	expect(result).toHaveProperty("default", "A");
});
