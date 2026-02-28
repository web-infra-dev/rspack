/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: { type: "assign-properties", name: ["process", "env"] }
	}
};
