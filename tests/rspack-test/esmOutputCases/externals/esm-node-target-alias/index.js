import { resolve } from "node:fs";
import { parse } from "node:url";
import { cjsResolve, cjsParse } from "./cjs-consumer.cjs";

it("should use aliased external request with correct external type", async () => {
	const main = await import(/* webpackIgnore: true */ "./main.mjs");
	const nodePath = await import(/* webpackIgnore: true */ "node:path");

	// ESM import of "module" external — keeps module type, uses aliased request
	expect(resolve).toBe(nodePath.resolve);
	expect(main.resolve).toBe(nodePath.resolve);

	// ESM import of "module-import" external — same target, also keeps module type
	expect(parse).toBe(nodePath.parse);
	expect(main.parse).toBe(nodePath.parse);

	// CJS require of "module" external — downgraded to node-commonjs
	expect(cjsResolve).toBe(nodePath.resolve);
	expect(main.cjsResolve).toBe(nodePath.resolve);

	// CJS require of "module-import" external — downgraded to node-commonjs
	expect(cjsParse).toBe(nodePath.parse);
	expect(main.cjsParse).toBe(nodePath.parse);
});

export { resolve, parse, cjsResolve, cjsParse };
