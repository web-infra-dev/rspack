module.exports = {
	entry: { main: "./index.js", test: "./test" },
	devtool: "eval",
	output: {
		filename: "[name].js",
		devtoolNamespace: "blackalbum",
		devtoolModuleFilenameTemplate: info =>
			`webpack://${info.namespace}/${info.resourcePath}?steins_gaess=god&${info.allLoaders}`
	},
	module: {
		rules: [{ test: /\.scss$/, use: [{ loader: "sass-loader" }], type: "css" }]
	}
};
