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
			      "message": "  × Module build failed (from ../../../../../../Library/pnpm/store/v10/links/json-loader/0.5.7/f99110a3408bdeca875c7787972ec6622d4bfb5dcb9e1d981257b7852c216eed/node_modules/json-loader/index.js):  ╰─▶   × SyntaxError: Unexpected end of JSON input        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx        │     at xxx      ",
			      "moduleId": "../../../../../../Library/pnpm/store/v10/links/json-loader/0.5.7/f99110a3408bdeca875c7787972ec6622d4bfb5dcb9e1d981257b7852c216eed/node_modules/json-loader/index.js!./not-a-json.js",
			      "moduleIdentifier": "<HOME>/Library/pnpm/store/v10/links/json-loader/0.5.7/f99110a3408bdeca875c7787972ec6622d4bfb5dcb9e1d981257b7852c216eed/node_modules/json-loader/index.js!<TEST_ROOT>/fixtures/errors/not-a-json.js",
			      "moduleName": "../../../../../../Library/pnpm/store/v10/links/json-loader/0.5.7/f99110a3408bdeca875c7787972ec6622d4bfb5dcb9e1d981257b7852c216eed/node_modules/json-loader/index.js!./not-a-json.js",
			      "moduleTrace": Array [],
			      "stack": undefined,
			    },
			  ],
			  "warnings": Array [],
			}
		`);
	}
};
