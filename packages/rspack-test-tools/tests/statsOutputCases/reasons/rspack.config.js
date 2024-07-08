/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./index",
	stats: {
		all: false,
		modules: true,
		reasons: true,
	}
};
