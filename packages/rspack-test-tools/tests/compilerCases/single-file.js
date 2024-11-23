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
		expect(error.toString()).toMatchInlineSnapshot(
			`Error: Missing field \`field0\` on RawOutputOptions.clean on RawOptions.output`
		);
	}
};
