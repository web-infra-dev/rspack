/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: {
			type: "modern-module"
		}
	},
	optimization: {
		runtimeChunk: false,
		avoidEntryIife: true
	}
};
