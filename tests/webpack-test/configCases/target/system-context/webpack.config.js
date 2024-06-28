/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: {
			type: "system"
		}
	},
	node: {
		__dirname: false,
		__filename: false
	}
};
