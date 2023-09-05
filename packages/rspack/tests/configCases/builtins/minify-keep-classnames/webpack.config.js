module.exports = {
	entry: {
		entry: "./entry.js",
		main: "./main.js"
	},
	builtins: {
		minifyOptions: {
			keepClassnames: true
		}
	},
	optimization: {
		minimize: true
	}
};
