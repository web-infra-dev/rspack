/** @type {import("@rspack/core").Configuration} */
module.exports = {
	// DIFF: no css file is generated in node target
	// target: ["web", "node"],
	target: "web",
	mode: "development",
	experiments: {
		css: true,
		outputModule: true
	}
};
