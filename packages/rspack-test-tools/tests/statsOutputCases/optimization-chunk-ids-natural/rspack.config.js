/** @type {import('@rspack/core').Configuration} */
module.exports = {
	mode: "production",
	entry: {
		e1: "./e1",
		e2: "./e2"
	},
	stats: {
		all: false,
		reasons: true,
		chunks: true,
		entrypoints: true,
		chunkGroups: true,
		errors: true,
	},
	optimization: { chunkIds: 'natural' },
};
