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
		      "formatted": "  ⚠ Error: test splice\\n  │     at <cwd>packages/rspack-test-tools/tests/errorCases/warning-test-splice-1.js:10:41\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:745:65\\n",
		      "message": "  ⚠ Error: test splice\\n  │     at <cwd>packages/rspack-test-tools/tests/errorCases/warning-test-splice-1.js:10:41\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:745:65\\n",
		    },
		  ],
		}
	`);
	}
};
