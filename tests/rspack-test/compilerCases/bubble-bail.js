let error;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description:
		"should bubble up errors when wrapped in a promise and bail is true (empty dependency)",
	options(context) {
		return {
			context: context.getSource(),
			mode: "production",
			entry: "./missing-file",
			output: {
				filename: "bundle.js"
			},
			bail: true
		};
	},
	async build(_, compiler) {
		try {
			await new Promise((resolve, reject) => {
				compiler.run((err, stats) => {
					if (err) {
						reject(err);
					}
					if (stats !== undefined && "errors" in stats) {
						reject(err);
					} else {
						resolve();
					}
				});
			});
		} catch (err) {
			error = err;
		}
	},
	async check() {
		expect(error).toBeTruthy();
		expect(error.toString()).toMatchInlineSnapshot(`
		Error:   × Module not found: Can't resolve './nonexistentfile' in '<TEST_ROOT>/fixtures'
		   ╭─[3:4]
		 1 │ module.exports = function b() {
		 2 │     /* eslint-disable node/no-missing-require */
		 3 │     require("./nonexistentfile");
		   ·     ────────────────────────────
		 4 │     return "This is a missing file";
		 5 │ };
		   ╰────
	`);
	}
};
