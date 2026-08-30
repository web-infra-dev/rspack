"use strict";

const [major, minor] = process.versions.node.split(".").map(Number);
module.exports = function filter(config) {
	return config.mode !== "development" && (major > 16 || (major === 16 && minor >= 11));
};
