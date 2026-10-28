/** @type {import("../../../..").TConfigCaseConfig} */
export default {
	findBundle: function (i, options) {
		if (i === 2) {
			return ["./bundle2.js"];
		}
	}
};
