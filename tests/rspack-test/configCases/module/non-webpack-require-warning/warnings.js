"use strict";

const inCacheTest = (options) => {
	if (Array.isArray(options)) {
		return options.some((o) => o.cache);
	}
	return options.cache;
};

module.exports = (options) => {
	if (inCacheTest(options)) {
		// We will pre-compile twice, and the module cache will result in no warnings in the stats during the third compilation
		return [];
	}
	return [
		// DIFF: rspack does not throw warning for __non_webpack_require__
		// [/__non_webpack_require__ is only allowed in target node/]
	];
};
