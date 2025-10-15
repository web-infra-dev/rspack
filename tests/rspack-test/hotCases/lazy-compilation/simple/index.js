it("should compile to lazy imported module", async () => {
	const done = err => (err ? reject(err) : resolve());
	let resolved;
	const promise = import("./module").then(r => (resolved = r));
	let generation = 0;
	module.hot.accept("./module", () => {
		generation++;
	});
	expect(resolved).toBe(undefined);
	await new Promise(resolve => setTimeout(resolve, 1000));
	expect(generation).toBe(0);
	await NEXT_HMR();
	let result = await promise;
	expect(result).toHaveProperty("default", 42);
	expect(generation).toBe(0);
	await NEXT_HMR();
	expect(result).toHaveProperty("default", 42);
	expect(generation).toBe(1);
	result = await import("./module");
	expect(result).toHaveProperty("default", 43);
	expect(generation).toBe(1);
	module.hot.accept("./module", () => {
		generation += 10;
	});
	await NEXT_HMR();
	result = await import("./module");
	expect(result).toHaveProperty("default", 44);
	expect(generation).toBe(11);
});
