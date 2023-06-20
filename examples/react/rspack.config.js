/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: {
		main: "./src/index.jsx"
	},
	devtool: false,
	module: {
		rules: [
			{
				test: /\.(png|svg|jpg)$/,
				type: "asset/resource"
			}
		]
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		],
		treeShaking: true
	},
	optimization: {
		sideEffects: true
	}
};
module.exports = config;
