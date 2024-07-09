/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing proxy methods on errors: test shift&unshift",
	options() {
		return {
			entry: "./resolve-fail-esm",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap(
						"test shift and unshift",
						compilation => {
							compilation.errors.shift();
							compilation.errors.unshift(new Error("test unshift"));
						}
					);
				}
			]
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
		Object {
		  "errors": Array [
		    Object {
		      "message": "  × Error: test unshift\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n",
		      "stack": "Error: test unshift\\n    at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/error-test-shift.js:13:35)\\n    at next (<cwd>packages/rspack-lite-tapable/dist/index.js:530:25)\\n    at AsyncSeriesHook.callAsyncStageRange (<cwd>packages/rspack-lite-tapable/dist/index.js:550:9)\\n    at AsyncSeriesHook.callAsync (<cwd>packages/rspack-lite-tapable/dist/index.js:88:21)\\n    at <cwd>packages/rspack/dist/Compiler.js:466:41\\n    at <cwd>packages/rspack/dist/Compiler.js:533:23",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
