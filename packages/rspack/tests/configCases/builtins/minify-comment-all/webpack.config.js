module.exports = {
	entry: {
		a: "./a",
		main: "./index"
	},
	builtins: {
		minifyOptions: {
			comments: "all"
		}
	},
	optimization: {
		minimize: true
	}
};
