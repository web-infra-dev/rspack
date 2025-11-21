// Avoid errors because of self-signed certificate
process.env["NODE_TLS_REJECT_UNAUTHORIZED"] = 0;

it("should compile to lazy imported module", async () => {
	let resolved;
	const promise = import("./module").then(r => (resolved = r));
	let generation = 0;
	import.meta.webpackHot.accept("./module", () => {
		generation++;
	});
	expect(resolved).toBe(undefined);
	expect(generation).toBe(0);
	await NEXT_HMR();
	const result = await promise;
	expect(result).toHaveProperty("default", 42);
	expect(generation).toBe(0);
	await NEXT_HMR();
	expect(result).toHaveProperty("default", 42);
	expect(generation).toBe(1);
	const m = await import("./module");
	expect(m).toHaveProperty("default", 43);
});
