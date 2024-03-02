module.exports = {
	target: "node",
	module: {
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
