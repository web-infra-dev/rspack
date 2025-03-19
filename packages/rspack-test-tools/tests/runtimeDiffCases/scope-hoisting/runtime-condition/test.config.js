/** @type {import("../../../../dist").TDiffCaseConfig} */
module.exports = {
	renameModule: raw => {
		// remove hash for concated module identifier
		return raw.split("|").slice(0, -1).join("|");
	},
	modules: true,
	files: ["shared.js", "a.js", "b.js"]
};
