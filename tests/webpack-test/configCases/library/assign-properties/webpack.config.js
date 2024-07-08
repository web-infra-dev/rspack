/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: ["process", "env"],
		libraryTarget: "assign-properties"
	}
};
