module.exports = {
	mode: "development",
	resolve: {
		byDependency: {
			esm: {
				extensions: [".bar", "..."]
			}
		}
	},
	experiments: {
		rspackFuture: {
			newResolver: true
		}
	}
};
