/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: 'development',
	context: __dirname,
	entry: {
		main: './src/index.js'
	},
	target: ['web', 'es5'],
	builtins: {
		html: [
			{
				template: './index.html'
			}
		]
	}
}
