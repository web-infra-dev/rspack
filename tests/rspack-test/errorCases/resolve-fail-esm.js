/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
module.exports = {
	description: "should emit warnings for resolve failure in esm",
	options() {
		return {
			entry: "./resolve-fail-esm"
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
			Object {
			  "errors": Array [],
			  "warnings": Array [],
			}
		`);
	}
};
