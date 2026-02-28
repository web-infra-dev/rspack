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
			},
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	output: {
		filename: "[name].js"
	}
};
