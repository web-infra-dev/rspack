/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "persistent cache storage.maxGenerations must be a non-negative integer",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./a",
			cache: {
				type: "persistent",
				storage: {
					type: "filesystem",
					maxGenerations: -1
				}
			}
		};
	},
	async build(_, compiler) {
		await new Promise(resolve => {
			compiler.run(() => resolve());
		});
	},
	async check({ context, name }) {
		const errors = context.getError(name);
		expect(Array.isArray(errors)).toBeTruthy();
		expect(errors.length).toBe(1);
		expect(errors[0].toString()).toContain(
			'Invalid Rspack configuration: "cache.storage.maxGenerations" must be an integer between 0 and 4294967295, get `-1`.'
		);
		context.clearError(name);
	}
};
