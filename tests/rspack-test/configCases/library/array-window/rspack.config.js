/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	output: {
		library: { type: "window", name: ["a", "b"] }
	}
};
