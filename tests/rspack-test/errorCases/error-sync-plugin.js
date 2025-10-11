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
		      "message": "  × Error: foo\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │ \\n",
		      "stack": "Error:   × Error: foo\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │     at xxx\\n  │ \\n",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
