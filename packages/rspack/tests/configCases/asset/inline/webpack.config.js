module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/inline"
			}
		]
	}
};
