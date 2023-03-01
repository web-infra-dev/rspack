module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.(svg|png)$/,
				type: "asset"
			}
		]
	}
};
