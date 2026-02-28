/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	output: {
		library: { type: "this", name: ["a", "b"] },
		environment: {
			arrowFunction: false
		}
	}
};
