/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	entry: "./index.js",
	experiments: {
		outputModule: true
	},
	output: {
		chunkFormat: "module",
		library: {
			type: "module"
		}
	}
};
