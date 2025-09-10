/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	findBundle: function (i, options) {
		return `bundle.${i}.js`;
	}
};
