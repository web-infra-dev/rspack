let error;

/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
	description:
		"should bubble up errors when wrapped in a promise and bail is true (empty dependency)",
	options(context) {
		return {
			context: context.getSource(),
			mode: "production",
			entry: "./empty-dependency",
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
			"Error:   × Empty dependency: Expected a non-empty request
			   ╭─[1:1]
			 1 │ module.exports = function b() {
			 2 │     /* eslint-disable node/no-missing-require */ require(\\"\\");
			   ·                                                  ───────────
			 3 │     return \\"This is an empty dependency\\";
			 4 │ };
			   ╰────
			"
		`);
	}
};
