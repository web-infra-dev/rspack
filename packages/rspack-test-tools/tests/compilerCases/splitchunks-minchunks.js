/** @type {import('../..').TCompilerCaseConfig} */
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
	async check(context) {
		const errors = context.getError('compiler/splitchunks-minchunks');
		expect(Array.isArray(errors)).toBeTruthy();
		expect(errors.length).toBe(1);
		expect(errors[0].toString()).toContain(
			'Number must be greater than or equal to 1 at "optimization.splitChunks.minChunks"'
		);
		context.clearError('compiler/splitchunks-minchunks');
	}
};
