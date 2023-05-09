/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: "development",
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js",
	},
	builtins: {
		html: [
			{
				template: "./index.html",
			},
		],
		treeShaking: true,
	},
};
module.exports = config;
