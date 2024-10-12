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
		      "stack": "Error: test splice\\n    at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/error-test-splice-1.js:<line>:<col>)\\n    at next (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at AsyncSeriesHook.callAsyncStageRange (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at AsyncSeriesHook.callAsync (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at <cwd>packages/rspack/dist/index.js:<line>:<col>\\n    at <cwd>packages/rspack/dist/index.js:<line>:<col>",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
