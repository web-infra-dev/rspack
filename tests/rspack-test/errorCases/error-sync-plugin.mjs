import throwErrorPlugin from "../fixtures/errors/throw-error-plugin.js";

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
			      "stack": "Error:   × Error: foo  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │ ",
			    },
			  ],
			  "warnings": Array [],
			}
		`);
	}
};
