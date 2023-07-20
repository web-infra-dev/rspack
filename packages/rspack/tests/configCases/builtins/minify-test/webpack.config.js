module.exports = {
	entry: {
		a: "./a",
		a2: "./a2",
		b: "./b",
		main: "./index"
	},
	builtins: {
		minifyOptions: {
			test: [/a\d?\.js/],
			exclude: [/a\.js/]
		}
	},
	optimization: {
		minimize: true
	}
};
