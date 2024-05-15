/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	findBundle: function (i) {
		if (i === 0) return ["main.js"];
		return ["main.mjs"];
	}
};
