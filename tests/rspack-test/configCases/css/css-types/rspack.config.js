/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	mode: "development",
	module: {
		rules: [
			{
				test: /\.css$/i,
				type: "css"
			},
			{
				test: /\.local\.css$/i,
				type: "css/module"
			},
			{
				test: /\.global\.css$/i,
				// MAYBE: support css/global
				// type: "css/global"
				type: "css/auto"
			},
			{
				test: /\.auto\.css$/i,
				type: "css/auto"
			},
			{
				test: /\.modules\.css$/i,
				type: "css/auto"
			}
		]
	},
	experiments: {
		css: true
	}
};
