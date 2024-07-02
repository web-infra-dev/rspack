/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
	description: "Testing proxy methods on warnings: test splice 1",
	options() {
		return {
			entry: "./require.main.require",
			plugins: [
				compiler => {
					compiler.hooks.afterCompile.tap("test splice", compilation => {
						compilation.warnings.splice(0, 1, new Error("test splice"));
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
		      "message": "  ⚠ Error: test splice\\n  │     at Object.fn (<cwd>packages/rspack-test-tools/tests/errorCases/warning-test-splice-1.js:10:41)\\n  │     at next (<cwd>packages/rspack-lite-tapable/dist/index.js:517:25)\\n  │     at AsyncSeriesHook.callAsyncStageRange (<cwd>packages/rspack-lite-tapable/dist/index.js:537:9)\\n  │     at AsyncSeriesHook.callAsync (<cwd>packages/rspack-lite-tapable/dist/index.js:75:21)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:463:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:530:23\\n",
		    },
		  ],
		}
	`);
	}
};
