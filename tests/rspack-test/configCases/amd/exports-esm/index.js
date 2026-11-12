import * as mixed from './mixed.js';

it("should pass the module exports object to require(['exports']) in an ESM module", async () => {
	const value = await mixed.ready;

	expect(value).toBe(1);
	expect(globalThis.__amdEsmExports).toBe(mixed);

	delete globalThis.__amdEsmExports;
});
