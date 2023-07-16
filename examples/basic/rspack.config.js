/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js"
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
