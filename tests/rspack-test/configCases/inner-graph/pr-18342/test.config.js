const findOutputFiles = require("@rspack/test-tools/helper/legacy/findOutputFiles");

module.exports = {
	findBundle(_, options) {
		const files = findOutputFiles(options, /^entry/);
		return files;
	}
};
