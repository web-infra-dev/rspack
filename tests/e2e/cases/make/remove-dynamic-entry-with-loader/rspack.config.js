const rspack = require("@rspack/core");

let first = true;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: async () => {
		if (first) {
			return {
				main: {
					import: "./loader.js!./src/index1.js",
				},
				main2: {
					import: "./loader.js!./src/index2.js"
				}
			}
		} else {
			return {
				main: {
					import: "./loader.js!./src/index1.js",
				},
			}
		}
	},
	context: __dirname,
	mode: "development",
	plugins: [
		new rspack.HtmlRspackPlugin(),
		function (compiler) {
			compiler.hooks.done.tap('t', () => {
				first = false;
			})
		}
	],
	devServer: {
		hot: true
	},
};
