/** @type {import("../../../../").TDiffCaseConfig} */
module.exports = {
	modules: true,
	runtimeModules: false,
	files: [
		"js.js",
		"mjs.js"
	],
	errors: true,
	replacements: [
		{
			from: /"[\s\S]*Module parse failed:[\s\S]*Top-level-await is only supported in EcmaScript Modules[\s\S]*"/g,
			to: "\"Module parse failed: Top-level-await is only supported in EcmaScript Modules\"",
		},
	],
};
