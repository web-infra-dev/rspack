// @ts-nocheck

module.exports = function supportsRequireInModule() {

	return !!require("module").createRequire;
};
