module.exports = {
	entry: {
		a: "./a",
		b: "./b",
		main: "./index"
	},
	builtins: {
		minifyOptions: {
			include: [/a\.js/]
		}
	},
	optimization: {
		minimize: true
	}
};
