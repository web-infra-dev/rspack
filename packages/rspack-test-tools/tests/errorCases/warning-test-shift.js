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
		      "message": "  ⚠ Error: test unshift\\n  │     at <cwd>packages/rspack-test-tools/tests/errorCases/warning-test-shift.js:13:37\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:464:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:531:65\\n",
		    },
		  ],
		}
	`);
	}
};
