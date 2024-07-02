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
		      "message": "  ⚠ Error: test unshift\\n  │     at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/warning-test-shift.js:13:37)\\n  │     at next (<cwd>packages/rspack-lite-tapable/dist/index.js:517:25)\\n  │     at AsyncSeriesHook.callAsyncStageRange (<cwd>packages/rspack-lite-tapable/dist/index.js:537:9)\\n  │     at AsyncSeriesHook.callAsync (<cwd>packages/rspack-lite-tapable/dist/index.js:75:21)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:463:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:530:23\\n",
		    },
		  ],
		}
	`);
	}
};
