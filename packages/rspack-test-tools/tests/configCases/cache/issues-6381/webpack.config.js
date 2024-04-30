/** @type {import("@rspack/core").Configuration} */
module.exports = {
	cache: true,
  snapshot: {
		module: {
			timestamp: true,
		},
		resolve: {
			timestamp: true,
		},
  },
};
