/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.jsx"
	},
	module: {
		rules: [
		]
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		],
		treeShaking: true,
		react: {
			refresh: false
		}
	},
	optimization: {
		sideEffects: true
	},
	target: ['web', 'es5']
};
module.exports = config;
