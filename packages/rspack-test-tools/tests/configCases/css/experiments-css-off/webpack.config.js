module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css"
			}
		]
	},
	bail: true,
	experiments: {
		css: false
	}
};
