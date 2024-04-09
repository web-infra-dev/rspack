module.exports = {
	entry: {
		index: {
			import: ["./index.js"]
		}
	},
	optimization: {
		providedExports: true,
		usedExports: "global"
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
