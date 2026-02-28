/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: { type: "system", name: "named-system-module-[name]" }
	},
	node: {
		__dirname: false,
		__filename: false
	}
};
