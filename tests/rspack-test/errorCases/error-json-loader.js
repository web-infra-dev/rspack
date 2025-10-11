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
		      "message": "  × Module build failed:\\n  ╰─▶   × SyntaxError: Unexpected end of JSON input\\n        │     at xxx\\n        │     at xxx\\n        │     at xxx\\n        │     at xxx\\n        │     at xxx\\n        │     at xxx\\n        │     at xxx\\n        │     at xxx\\n      \\n",
		      "moduleId": "../../../../node_modules/<PNPM_INNER>/json-loader/index.js!./not-a-json.js",
		      "moduleIdentifier": "<ROOT>/node_modules/<PNPM_INNER>/json-loader/index.js!<TEST_TOOLS_ROOT>/fixtures/errors/not-a-json.js",
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
