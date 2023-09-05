module.exports = {
	entry: {
		entry: "./entry.js",
		main: "./main.js"
	},
	builtins: {
		minifyOptions: {
			keepFnames: true
		}
	},
	optimization: {
		minimize: true
	}
};
