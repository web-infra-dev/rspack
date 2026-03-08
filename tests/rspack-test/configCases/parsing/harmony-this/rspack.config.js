/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		parser: {
			javascript: {
				strictThisContextOnImports: true
			}
		}
	},
	optimization: {
		concatenateModules: false
	}
};
