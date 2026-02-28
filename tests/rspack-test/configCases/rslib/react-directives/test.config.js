/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	findBundle: function (i, options) {
		if (i === 3) {
			return ["./bundle3.js"];
		}
	}
};
