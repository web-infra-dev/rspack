import { resolve } from "node:fs";
import { cjsResolve } from "./cjs-consumer.cjs";

it("should use aliased external request with correct external type", async () => {
	const main = await import(/* webpackIgnore: true */ "./main.mjs");
	const nodePath = await import(/* webpackIgnore: true */ "node:path");

	// "node:fs" is aliased to "node:path" via externals config
	// ESM import should use "module" external type
	expect(resolve).toBe(nodePath.resolve);
	expect(main.resolve).toBe(nodePath.resolve);

	// CJS require should be downgraded to "node-commonjs" external type
	expect(cjsResolve).toBe(nodePath.resolve);
	expect(main.cjsResolve).toBe(nodePath.resolve);
});

export { resolve, cjsResolve };
