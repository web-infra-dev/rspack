const p = require('./plugins/scheme')
/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: 'development',
	context: __dirname,
	entry: {
		main: './src/index.js'
	},
	builtins: {
		html: [
			{
				template: './index.html'
			}
		]
	},
	plugins: [new p()]
}
