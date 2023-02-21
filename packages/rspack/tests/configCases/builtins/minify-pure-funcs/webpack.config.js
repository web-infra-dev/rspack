module.exports = {
	builtins: {
		minifyOptions: {
			pureFuncs: ["console.debug", "console.warn"]
		}
	},
	optimization: {
		minimize: true
	}
};
