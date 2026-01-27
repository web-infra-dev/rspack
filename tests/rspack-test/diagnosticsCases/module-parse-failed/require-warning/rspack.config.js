/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		parser: {
			javascript: {
				requireAlias: true,
				requireAsExpression: true
			}
		}
	}
};
