/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css"
			}
		]
	},
	bail: true,
	experiments: {
		css: false
	}
};
