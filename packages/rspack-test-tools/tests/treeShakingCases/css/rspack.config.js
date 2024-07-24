/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		parser: {
			'css/auto': {
				namedExports: false,
			}
		},
		rules: [
			{
				test: /\.module\.css/,
				type: "css/auto"
			}
		]
	},
	experiments: {
		css: true
	}
};
