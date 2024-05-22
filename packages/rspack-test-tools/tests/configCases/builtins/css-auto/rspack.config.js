/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto",
				generator: {
					exportsOnly: false,
				}
			}
		]
	}
};
