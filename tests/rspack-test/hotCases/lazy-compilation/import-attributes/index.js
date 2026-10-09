import generation from "./generation.js";

import.meta.webpackHot.accept("./generation.js");

it("should keep import attributes for lazy imported module", async () => {
	let resolved;
	const promise = import("./data/a.json", { with: { type: "json" } }).then(
		r => (resolved = r)
	);
	const start = generation;
	expect(resolved).toBe(undefined);
	await NEXT_HMR();
	const result = await promise;
	expect(result.default).toEqual({ value: "a" });
	expect(generation).toBe(start + 1);
});

it("should keep import attributes for lazy imported context element", async () => {
	const key = "b";
	let resolved;
	const promise = import(`./data/${key}.json`, { with: { type: "json" } }).then(
		r => (resolved = r)
	);
	const start = generation;
	expect(resolved).toBe(undefined);
	await NEXT_HMR();
	const result = await promise;
	expect(result.default).toEqual({ value: "b" });
	expect(generation).toBe(start + 1);
});
