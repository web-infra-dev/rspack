/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	findBundle: function (i, options) {
		if (i===2) {
			return ["./bundle2.mjs"];
		}
	}
};
