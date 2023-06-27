/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	output: {
		library: ["a", "b"],
		libraryTarget: "window"
	}
};
