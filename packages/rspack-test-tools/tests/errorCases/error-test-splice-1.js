/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing proxy methods on errors: test splice 1",
	options() {
		return {
			entry: "./resolve-fail-esm",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("test splice", compilation => {
						compilation.errors.splice(0, 1, new Error("test splice"));
					});
				}
			]
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
		Object {
		  "errors": Array [
		    Object {
		      "message": "  × Error: test splice\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n",
		      "moduleTrace": Array [],
		      "stack": "Error: test splice\\n    at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/error-test-splice-1.js:10:39)\\n    at next (<cwd>packages/rspack-lite-tapable/src/index.ts:773:10)\\n    at AsyncSeriesHook.callAsyncStageRange (<cwd>packages/rspack-lite-tapable/src/index.ts:790:3)\\n    at AsyncSeriesHook.callAsync (<cwd>packages/rspack-lite-tapable/src/index.ts:218:15)\\n    at <cwd>packages/rspack/src/Compiler.ts:613:29\\n    at <cwd>packages/rspack/src/Compiler.ts:660:15",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
