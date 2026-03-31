const path = require('path')

/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
module.exports = {
	description: 'should error when preserveModules entry module is used in multiple entries',
	options() {
		const context = path.resolve(
			__dirname,
			'../fixtures/errors/preserve-modules-multiple-entries',
		)

		return {
			context,
			mode: 'production',
			target: 'async-node',
			entry: {
				a: './src/shared.js',
				b: './src/shared.js',
			},
			output: {
				path: path.resolve(context, 'dist'),
				filename: '[name].mjs',
				module: true,
				library: {
					type: 'modern-module',
					preserveModules: path.resolve(context, 'src'),
				},
			},
			optimization: {
				minimize: false,
				moduleIds: 'named',
				chunkIds: 'named',
				runtimeChunk: 'single',
			},
		}
	},
	async check(diagnostics) {
		expect(diagnostics.errors).toHaveLength(1)
		expect(diagnostics.warnings).toHaveLength(0)
		expect(diagnostics.errors[0].message).toContain('used in multiple entries')
		expect(diagnostics.errors[0].message).toContain('preserveModules')
		expect(diagnostics.errors[0].message).toContain('a')
		expect(diagnostics.errors[0].message).toContain('b')
	},
}
