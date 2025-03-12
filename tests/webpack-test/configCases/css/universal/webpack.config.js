/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: ["web", "node"],
	mode: "development",
	experiments: {
		css: true,
		outputModule: true
	}
};
