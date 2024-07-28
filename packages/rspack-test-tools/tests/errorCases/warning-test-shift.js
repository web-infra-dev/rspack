/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing proxy methods on warnings: test shift&unshift",
	options() {
		return {
			entry: "./require.main.require",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap(
						"test shift and unshift",
						compilation => {
							compilation.warnings.shift();
							compilation.warnings.unshift(new Error("test unshift"));
						}
					);
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
		      "message": "  ⚠ Error: test unshift\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n",
		      "moduleTrace": Array [],
		      "stack": "Error: test unshift\\n    at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/warning-test-shift.js:13:37)\\n    at next (<cwd>packages/rspack-lite-tapable/src/index.ts:773:10)\\n    at AsyncSeriesHook.callAsyncStageRange (<cwd>packages/rspack-lite-tapable/src/index.ts:790:3)\\n    at AsyncSeriesHook.callAsync (<cwd>packages/rspack-lite-tapable/src/index.ts:218:15)\\n    at <cwd>packages/rspack/src/Compiler.ts:613:29\\n    at <cwd>packages/rspack/src/Compiler.ts:660:15",
		    },
		  ],
		}
	`);
	}
};
