module.exports = {
	entry: {
		main: "./index.js",
		css: "./css"
	},
	experiments: {
		css: true
	},
	builtins: {
		css: {
			modules: {
				localsConvention: "camelCase"
			}
		}
	}
};
