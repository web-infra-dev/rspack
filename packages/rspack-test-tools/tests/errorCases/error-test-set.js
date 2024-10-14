/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing set errors",
	options() {
		return {
			entry: "./resolve-fail-esm",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("set errors", compilation => {
						compilation.errors = [
							new Error("error 1"),
							new Error("error 2")
						];
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
		      "message": "  × Error: error 1\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n",
		      "moduleTrace": Array [],
		      "stack": "Error: error 1\\n    at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/error-test-set.js:<line>:<col>)\\n    at next (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at AsyncSeriesHook.callAsyncStageRange (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at AsyncSeriesHook.callAsync (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at <cwd>packages/rspack/dist/index.js:<line>:<col>\\n    at <cwd>packages/rspack/dist/index.js:<line>:<col>",
		    },
		    Object {
		      "message": "  × Error: error 2\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n",
		      "moduleTrace": Array [],
		      "stack": "Error: error 2\\n    at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/error-test-set.js:<line>:<col>)\\n    at next (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at AsyncSeriesHook.callAsyncStageRange (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at AsyncSeriesHook.callAsync (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at <cwd>packages/rspack/dist/index.js:<line>:<col>\\n    at <cwd>packages/rspack/dist/index.js:<line>:<col>",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
