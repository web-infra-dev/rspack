/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	// TODO: cache group test block_on
	concurrent: false,
	findBundle: function (i, options) {
		return [`./${options.name}-main.js`];
	}
};
