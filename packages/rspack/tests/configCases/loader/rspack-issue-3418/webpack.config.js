module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /lib\.js$/,
				loader: "./loader.js"
			}
		]
	}
};
