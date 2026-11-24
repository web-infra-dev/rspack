import throwErrorPlugin from "../fixtures/errors/throw-error-plugin.mjs";

/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
export default {
	description: "should thrown sync error from plugin",
	options() {
		return {
			entry: "./no-errors-deprecate",
			plugins: [throwErrorPlugin]
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "code": "GenericFailure",
			      "message": "  × Error: foo  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │ ",
			      "stack": "Error: foo    at Object.fn (<ROOT>/tests/rspack-test/fixtures/errors/throw-error-plugin.mjs<LINE_COL>)    at SyncHook.callAsyncStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at SyncHook.callStageRange (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at QueriedHook.call (<RSPACK_ROOT>/dist/index.js<LINE_COL>)    at <RSPACK_ROOT>/dist/index.js<LINE_COL>    at last.function (<RSPACK_ROOT>/dist/index.js<LINE_COL>)",
			    },
			  ],
			  "warnings": Array [],
			}
		`);
	}
};
