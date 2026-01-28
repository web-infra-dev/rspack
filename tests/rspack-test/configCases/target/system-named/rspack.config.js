/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: {
			name: "named-system-module",
			type: "system"
		}
	},
	node: {
		__dirname: false,
		__filename: false
	}
};
