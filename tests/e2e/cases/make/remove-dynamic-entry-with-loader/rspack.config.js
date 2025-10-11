const rspack = require("@rspack/core");

const sharedObj = {
	useFullEntry: true
};

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: async () => {
		if (sharedObj.useFullEntry) {
			return {
				main: {
					import: "./loader.js!./src/index1.js"
				},
				main2: {
					import: "./loader.js!./src/index2.js"
				}
			};
		} else {
			return {
				main: {
					import: "./loader.js!./src/index1.js"
				}
			};
		}
	},
	context: __dirname,
	mode: "development",
	plugins: [
		new rspack.HtmlRspackPlugin(),
		function (compiler) {
			compiler.__sharedObj = sharedObj;
		}
	],
	devServer: {
		hot: true
	}
};
