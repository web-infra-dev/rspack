/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	findBundle: function () {
		return Array.from({ length: 10 }, (_, i) => `./bundle${i + 1}.js`);
	}
};
