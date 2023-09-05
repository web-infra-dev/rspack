module.exports = {
	entry: "./index.js",
	resolve: {
		exportsFields: ["a", "b"]
	},
	experiments: {
		rspackFuture: {
			newResolver: true
		}
	}
};
