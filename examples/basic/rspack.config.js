/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: {
		main: './src/index.js'
	},
	builtins: {
		html: [
			{
			}
		]
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				use: './loader.js'
			}
		]
	}
}
