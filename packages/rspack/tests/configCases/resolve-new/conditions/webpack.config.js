module.exports = {
	entry: "./index.js",
	resolve: {
		conditionNames: ["pack"]
	},
	experiments: {
		rspackFuture: {
			newResolver: true
		}
	}
};
