/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	writeStatsJson: true,
	findBundle: function (i, options) {
		return ["main-1.js", "main-cde42ecf.js", "main-f5c11e54.js"];
	}
};
