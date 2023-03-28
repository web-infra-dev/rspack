/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	entry: {
		main: './index.js'
	},
	target: ["node", "es5"],
	builtins: {
		html: [
			{
				template: './index.html'
			}
		]
	}
}
