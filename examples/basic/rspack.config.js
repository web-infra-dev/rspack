/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: 'development',
	target: ['web', 'es5'],
	context: __dirname,
	entry: {
		main: './src/index.js'
	},
	optimization: {
		sideEffects: true
	},
	builtins: {
		treeShaking: true,
		html: [
			{
				template: './index.html'
			}
		]
	}
}
