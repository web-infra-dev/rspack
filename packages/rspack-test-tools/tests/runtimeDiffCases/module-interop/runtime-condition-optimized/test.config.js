/** @type {import("../../..").TDiffCaseConfig} */
module.exports = {
	modules: true,
	runtimeModules: false,
	files: [
		"shared.js",
		"a.js",
		"b.js",
		"c1.js",
		"c2.js",
		"ax.js",
		"bx.js",
		"cx1.js",
		"cx2.js",
		"d1.js",
		"d2.js"
	],
	renameModule: raw => {
		// remove hash for concated module identifier
		return raw.split("|").slice(0, -1).join("|");
	}
};
