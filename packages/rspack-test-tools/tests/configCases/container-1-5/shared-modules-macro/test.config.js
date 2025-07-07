module.exports = {
	description:
		"Tree-shaking macros for shared modules (CJS and ESM) in Module Federation",
	options(context) {
		return {
			experiments: {
				outputModule: false,
				rspackFuture: {
					bundlerInfo: {
						force: false
					}
				}
			},
			cache: false // Disable cache to avoid serialization issues
		};
	},
	diffStats: true,
	nonEsmThis: "(global || {})"
};
