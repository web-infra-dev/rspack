exports.buildCanonicalizedResource = function (resourcePath) {
	return `resource:${resourcePath}`;
};

exports.buildCanonicalString = function (resourcePath) {
	return this.buildCanonicalizedResource(resourcePath);
};

exports.usedExports = __webpack_exports_info__.usedExports;
