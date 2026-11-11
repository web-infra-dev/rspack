const fs = require("node:fs");
const path = require("node:path");

module.exports = {
	// Leave no link behind in the test temp directory.
	afterExecute(options) {
		fs.rmSync(path.join(options.context, "config", "optional-directory"), { force: true });
	}
};
