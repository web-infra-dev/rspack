module.exports = {
	entry: {
		asset: "./image.svg",
		style: "./main.css",
		main: "./main.js"
	},
	target: "web",
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset/resource"
			}
		]
	},
	output: {
		filename: "[name].js"
	},
	experiments: {
		css: true
	}
};
