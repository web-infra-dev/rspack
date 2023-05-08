module.exports = {
	builtins: {
		minifyOptions: {
			pureFuncs: ["console.error", "console.warn"]
		}
	},
	optimization: {
		minimize: true
	}
};
