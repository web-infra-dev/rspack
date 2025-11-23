const validateOutput = require("./validate-output");

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
	nonEsmThis: "(global || {})",
	afterBuild(context) {
		// Validate that PURE annotations and tree-shaking macros are in the output
		try {
			validateOutput(context.getDist());
			return Promise.resolve();
		} catch (err) {
			return Promise.reject(err);
		}
	}
};
