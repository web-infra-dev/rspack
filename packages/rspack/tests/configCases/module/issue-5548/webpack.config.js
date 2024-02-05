module.exports = {
	mode: "production",
	optimization: {
		minimize: false
	},
	module: {
		parser: {
			javascript: {
				dynamicImportMode: "eager"
			}
		}
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
