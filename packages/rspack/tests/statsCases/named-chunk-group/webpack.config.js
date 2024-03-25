/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./index",
	stats: {
		all: false,
		entrypoints: true,
		chunkGroups: true,
	}
};
