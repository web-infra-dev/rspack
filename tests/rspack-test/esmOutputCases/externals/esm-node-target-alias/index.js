import { resolve } from "node:fs";
import { parse } from "node:url";
import { cjsResolve, cjsParse } from "./cjs-consumer.cjs";

it("should use aliased external request with correct external type", async () => {
	const main = await import(/* webpackIgnore: true */ "./main.mjs");
	const nodePath = await import(/* webpackIgnore: true */ "node:path");
	const nodeUrl = await import(/* webpackIgnore: true */ "node:url");

	// ESM import of "module" external — keeps module type, uses aliased request
	expect(resolve).toBe(nodePath.resolve);
	expect(main.resolve).toBe(nodePath.resolve);

	// ESM import of "module-import" external — keeps module-import type
	expect(parse).toBe(nodeUrl.parse);
	expect(main.parse).toBe(nodeUrl.parse);

	// CJS require of "module" external — downgraded to node-commonjs
	expect(cjsResolve).toBe(nodePath.resolve);
	expect(main.cjsResolve).toBe(nodePath.resolve);

	// CJS require of "module-import" external — downgraded to node-commonjs
	expect(cjsParse).toBe(nodeUrl.parse);
	expect(main.cjsParse).toBe(nodeUrl.parse);
});

export { resolve, parse, cjsResolve, cjsParse };
