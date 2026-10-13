import generation from "./generation.js";

import.meta.webpackHot.accept("./generation.js");

it("should keep issuer for lazy imported module", async () => {
	let resolved;
	const promise = import("./data/issuer.txt").then(r => (resolved = r));
	const start = generation;
	expect(resolved).toBe(undefined);
	await new Promise(resolve => setTimeout(resolve, 1000));
	expect(generation).toBe(start);
	await NEXT_HMR();
	const result = await promise;
	expect(result.default.trim()).toBe("issuer");
	expect(generation).toBe(start + 1);
});

it("should keep issuer resolve options for lazy imported module", async () => {
	let resolved;
	const promise = import("./data/resolve").then(r => (resolved = r));
	const start = generation;
	expect(resolved).toBe(undefined);
	await new Promise(resolve => setTimeout(resolve, 1000));
	expect(generation).toBe(start);
	await NEXT_HMR();
	const result = await promise;
	expect(result.default.trim()).toBe("resolve");
	expect(generation).toBe(start + 1);
});
