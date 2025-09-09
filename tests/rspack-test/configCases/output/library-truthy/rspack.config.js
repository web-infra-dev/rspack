/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: {
			type: "modern-module"
		}
	},
	optimization: {
		avoidEntryIife: true
	}
};
