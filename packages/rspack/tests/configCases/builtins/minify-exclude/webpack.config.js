module.exports = {
	entry: {
		a: "./a",
		b: "./b",
		main: "./index"
	},
	builtins: {
		minifyOptions: {
			exclude: [/b\.js/]
		}
	},
	optimization: {
		minimize: true
	}
};
