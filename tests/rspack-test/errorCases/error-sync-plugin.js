/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
module.exports = {
	description: "should thrown sync error from plugin",
	options() {
		return {
			entry: "./no-errors-deprecate",
			plugins: [require("../fixtures/errors/throw-error-plugin")]
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "code": "GenericFailure",
			      "message": "  × Error: foo  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │ ",
			      "stack": "Error:   × Error: foo  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │     at xxx  │ ",
			    },
			  ],
			  "warnings": Array [],
			}
		`);
	}
};
