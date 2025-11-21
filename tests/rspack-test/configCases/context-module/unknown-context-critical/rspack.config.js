/** @type {import("@rspack/core").Configuration} */
module.exports = {
	amd: false,
	module: {
		parser: {
			javascript: {
				unknownContextCritical: true
			}
		}
	}
};
