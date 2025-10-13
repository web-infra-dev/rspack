/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "splitChunks.minChunks equals 0",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./a",
			optimization: {
				splitChunks: {
					minChunks: 0
				}
			}
		};
	},
	async build(_, compiler) {
		await new Promise(resolve => {
			compiler.run(() => {
				compiler.run(() => {
					resolve();
				});
			});
		});
	},
	async check({ context, name }) {
		const errors = context.getError(name);
		expect(Array.isArray(errors)).toBeTruthy();
		expect(errors.length).toBe(1);
		expect(errors[0].toString()).toContain(
			'Number must be greater or equal to 1 at "optimization.splitChunks.minChunks"'
		);
		context.clearError(name);
	}
};
