/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: {
			name: "MyLibrary",
			export: "default",
			type: "window"
		}
	}
};
