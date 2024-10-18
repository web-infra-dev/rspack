/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing set warnings",
	options() {
		return {
			entry: "./require.main.require",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("set warnings", compilation => {
						compilation.warnings = [
							new Error("warning 1"),
							new Error("warning 2")
						];
					});
				}
			]
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
		Object {
		  "errors": Array [],
		  "warnings": Array [
		    Object {
		      "message": "  ⚠ Error: warning 1\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n",
		      "moduleTrace": Array [],
		      "stack": "Error: warning 1\\n    at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/warning-test-set.js:<line>:<col>)\\n    at next (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at AsyncSeriesHook.callAsyncStageRange (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at AsyncSeriesHook.callAsync (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at <cwd>packages/rspack/dist/index.js:<line>:<col>\\n    at <cwd>packages/rspack/dist/index.js:<line>:<col>",
		    },
		    Object {
		      "message": "  ⚠ Error: warning 2\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n",
		      "moduleTrace": Array [],
		      "stack": "Error: warning 2\\n    at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/warning-test-set.js:<line>:<col>)\\n    at next (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at AsyncSeriesHook.callAsyncStageRange (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at AsyncSeriesHook.callAsync (<cwd>node_modules/.pnpm/@rspack+lite-tapable@1.0.1/node_modules/@rspack/lite-tapable/dist/index.js:<line>:<col>)\\n    at <cwd>packages/rspack/dist/index.js:<line>:<col>\\n    at <cwd>packages/rspack/dist/index.js:<line>:<col>",
		    },
		  ],
		}
	`);
	}
};
