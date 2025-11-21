import contextImport from "./context-import.js";
import generation from "./generation.js";

import.meta.webpackHot.accept("./generation.js");

for (const name of ["demo", "module"]) {
	it("should compile to lazy imported context element " + name, async () => {
		let resolved;
		const promise = contextImport(name)
			.then(r => (resolved = r));
		const start = generation;
		expect(resolved).toBe(undefined);
		await new Promise(resolve => setTimeout(resolve, 1000));
		expect(generation).toBe(start);
		await NEXT_HMR();
		const result = await promise;
		expect(result).toHaveProperty("default", name);
		expect(generation).toBe(start + 1);
	});
}
