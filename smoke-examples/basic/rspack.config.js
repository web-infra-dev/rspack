/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: 'development',
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	optimization: {
		sideEffects: true
	},
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: 'asset/resource'
			}
		]
	},
	builtins: {
		treeShaking: true,
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
