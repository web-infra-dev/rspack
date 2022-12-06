module.exports = {
	mode: "development",
	// output: {
	// 	assetModuleFilename: "images/[hash][ext]"
	// },
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	module: {
		rules: [
			// {
			// test: /\.(png|jpg|svg)$/,
			// type: "asset"
			//
		]
	}
};
