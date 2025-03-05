/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	writeStatsJson: true,
	findBundle: function (i, options) {
		return ["main~1.js", "main~2.js", "main~3.js", "main~400f55e3.js"];
	}
};
