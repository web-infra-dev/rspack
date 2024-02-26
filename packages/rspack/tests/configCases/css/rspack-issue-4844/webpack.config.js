module.exports = {
	entry: {
		main: "./index.js",
		css: "./css"
	},
	output: {
		filename: "[name].js"
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
