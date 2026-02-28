/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
module.exports = {
	description: "should emit error for json-loader when not json",
	options() {
		return {
			entry: "json-loader!./not-a-json.js"
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [
			    Object {
			      "code": "ModuleBuildError",
			      "message": "  × Module build failed (from ../../../../node_modules/<PNPM_INNER>/json-loader/index.js):  ╰─▶   × SyntaxError: Unexpected end of JSON input        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx      ",
			      "moduleId": "../../../../node_modules/<PNPM_INNER>/json-loader/index.js!./not-a-json.js",
			      "moduleIdentifier": "<ROOT>/node_modules/<PNPM_INNER>/json-loader/index.js!<TEST_ROOT>/fixtures/errors/not-a-json.js",
			      "moduleName": "../../../../node_modules/<PNPM_INNER>/json-loader/index.js!./not-a-json.js",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			  ],
			  "warnings": Array [],
			}
		`);
	}
};
