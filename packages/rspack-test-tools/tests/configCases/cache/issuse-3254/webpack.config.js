/** @type {import("@rspack/core").Configuration} */
module.exports = {
	cache: true,
	snapshot: {
		module: { timestamp: true, hash: true },
		resolve: {
			timestamp: true,
			hash: true
		}
	}
};
