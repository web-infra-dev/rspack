/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	target: 'node',
	entry: {
		main: './src/main.ts'
	},
	externalsType: 'commonjs',
	externals: ['@nestjs/common', '@nestjs/core']
}
