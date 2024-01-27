const _module = {
	rules: [{ test: /\.scss$/, use: [{ loader: "sass-loader" }], type: "css" }]
};
module.exports = () => [
	{
		entry: { main: "./index.js", test: "./test" },
		devtool: "eval",
		output: {
			devtoolNamespace: "blackalbum",
			devtoolModuleFilenameTemplate: info =>
				`webpack://${info.namespace}/${info.resourcePath}?steins_gaess=god&${info.allLoaders}`
		},

		module: _module
	}
];
