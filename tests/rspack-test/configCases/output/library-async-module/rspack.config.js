/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	entry: "./index.js",
	experiments: {
		},
	output: {
		module: true,
		chunkFormat: "module",
		library: {
			type: "module"
		}
	}
};
