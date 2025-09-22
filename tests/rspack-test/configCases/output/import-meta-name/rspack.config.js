/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		importMetaName: "pseudoImport.meta"
	},
	module: {
		parser: {
			javascript: {
				importMeta: false
			}
		}
	}
};
