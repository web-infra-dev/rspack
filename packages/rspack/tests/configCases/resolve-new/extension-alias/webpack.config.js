module.exports = {
	entry: "./index.js",
	resolve: {
		extensionAlias: {
			".mjs": [".mts"]
		}
	},
	experiments: {
		rspackFuture: {
			newResolver: true
		}
	}
};
