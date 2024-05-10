/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.module\.css$/,
				type: "css/module"
			}
		]
	}
};
