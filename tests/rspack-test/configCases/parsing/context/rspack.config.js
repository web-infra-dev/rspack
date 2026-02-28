/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		unknownContextRegExp: /^\.\//,
		unknownContextCritical: false,
		exprContextRegExp: /^\.\//,
		exprContextCritical: false
	}
};
