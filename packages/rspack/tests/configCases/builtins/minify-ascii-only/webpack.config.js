module.exports = {
	entry: {
		a: "./a",
		main: "./index"
	},
	builtins: {
		minifyOptions: {
			asciiOnly: true
		}
	},
	optimization: {
		minimize: true
	}
};
