/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /index\.js/,
				use: ["./import-loader.js", "./import-loader-2.js"]
			}
		]
	}
};
