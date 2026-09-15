var { supportDefaultAssignment } = require("@rspack/test-tools/helper/legacy/supportDefaultAssignment");

module.exports = function (config) {
	return supportDefaultAssignment();
};
