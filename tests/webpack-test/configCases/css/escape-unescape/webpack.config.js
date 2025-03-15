/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		target: "web",
		mode: "development",
		experiments: {
			css: true
		}
	},
	{
		target: "web",
		mode: "production",
		experiments: {
			css: true
		}
	}
];
