"use strict";

const isWindows = process.platform === "win32";

function skipTestOnWindows(reason) {
	if (isWindows) {
		test.skip(reason, () => {});
	}
	return isWindows;
}

module.exports.skipTestOnWindows = skipTestOnWindows;
