const fs = require("node:fs");
const path = require("node:path");

module.exports = {
	// CI collects the cache folders below the temp directory with a glob, which fails on a looping link.
	afterExecute(options) {
		for (const link of ["dangling", "loop"]) {
			fs.rmSync(path.join(options.context, "pages", link), { force: true });
		}
	}
};
