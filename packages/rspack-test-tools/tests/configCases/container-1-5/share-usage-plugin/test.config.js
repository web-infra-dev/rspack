module.exports = {
	description:
		"ShareUsagePlugin should accurately track used/unused exports for CJS, ESM, and local shared modules",
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
			cache: false // Disable cache to ensure fresh analysis
		};
	},
	diffStats: true,
	nonEsmThis: "(global || {})"
};
