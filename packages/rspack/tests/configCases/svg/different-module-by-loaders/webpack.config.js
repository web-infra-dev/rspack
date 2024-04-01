module.exports = {
	target: "node",
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			},
			"css": {
				exportsOnly: false,
			},
			"css/module": {
				exportsOnly: false,
			}
		},
		rules: [
			{
				test: /\.svg$/i,
				issuer: { not: [/\.css$/] },
				use: [{ loader: "file-loader", options: { name: "[name].[ext]" } }],
				type: "javascript/auto"
			},
			{
				test: /\.svg$/,
				issuer: { not: [/\.js$/] },
				type: "asset/inline"
			}
		]
	}
};
