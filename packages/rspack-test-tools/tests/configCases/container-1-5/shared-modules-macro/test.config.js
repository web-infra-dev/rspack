module.exports = {
	description: "Tree-shaking macros for shared modules (CJS and ESM) in Module Federation",
	options(context) {
		return {
			experiments: {
				outputModule: false
			}
		};
	},
	diffStats: true,
	nonEsmThis: "(global || {})"
};