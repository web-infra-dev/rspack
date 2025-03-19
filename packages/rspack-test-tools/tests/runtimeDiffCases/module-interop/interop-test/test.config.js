/** @type {import("../../../../").TDiffCaseConfig} */
module.exports = {
	modules: true,
	runtimeModules: false,
	files: ["js.js", "mjs.js"],
	errors: true,
	replacements: [
		{
			from: /throw new Error\("(.*)Module parse failed(.*)Top-level-await is only supported in EcmaScript Modules(.*)"\)/g,
			to: 'throw new Error("Module parse failed: Top-level-await is only supported in EcmaScript Modules")'
		}
	]
};
