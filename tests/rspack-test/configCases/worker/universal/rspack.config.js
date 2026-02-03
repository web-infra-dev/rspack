/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		name: "web",
		target: ["web", "node"],
		output: {
			module: true,
			filename: "web-[name].mjs"
		},
	}
];
