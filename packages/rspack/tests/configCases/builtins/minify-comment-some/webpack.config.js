module.exports = {
	entry: {
		a: "./a",
		main: "./index"
	},
	builtins: {
		minifyOptions: {
			comments: "some"
		}
	},
	optimization: {
		minimize: true
	}
};
