const validateSyntax = require("./validate-syntax");

module.exports = {
	description:
		"Validates syntax correctness of tree-shaking macros in shared modules",
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
			cache: false
		};
	},
	diffStats: true,
	nonEsmThis: "(global || {})",
	afterBuild(context) {
		// Validate syntax correctness after macro processing
		try {
			validateSyntax(context.getDist());
			return Promise.resolve();
		} catch (err) {
			return Promise.reject(err);
		}
	}
};
