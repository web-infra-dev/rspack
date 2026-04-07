import { resolve } from "node:fs";

it("should use aliased external request with correct external type", async () => {
	const main = await import(/* webpackIgnore: true */ "./main.mjs");
	const nodePath = await import(/* webpackIgnore: true */ "node:path");

	// "node:fs" is aliased to "node:path", so resolve should come from node:path
	expect(resolve).toBe(nodePath.resolve);
	expect(main.resolve).toBe(nodePath.resolve);
});

export { resolve };
