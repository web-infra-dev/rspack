import { run } from "../../../utils/test-utils";
import { existsSync } from "fs";
import { resolve } from "path";

describe("tsconfig ", () => {
	it("empty tsconfig", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, []);
		expect(stderr).toBeFalsy();
		expect(stdout).toMatchInlineSnapshot(`
		"[33mInvalid file object. JSON schema for the TypeScript compiler's configuration has been initialized using a file object that does not match the API schema.
		 - file.compilerOptions misses the property 'verbatimModuleSyntax'. Should be:
		   true
		   -> SWC warning more info see: https://swc.rs/docs/migrating-from-tsc#esmoduleinterop-true
		   Do not transform or elide any imports or exports not marked as type-only, ensuring they are written in the output file's format based on the 'module' setting. [0m
		Time: [1m191[39m[22mms"
	`);
		expect(exitCode).toBe(0);
	});
});
