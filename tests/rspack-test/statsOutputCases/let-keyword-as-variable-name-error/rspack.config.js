/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./index",
	stats: "errors-warnings",
	module: {
		rules: [
			{
				test: /.js/,
				type: "javascript/esm"
			}
		]
	}
};
