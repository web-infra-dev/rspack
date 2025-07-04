// CommonJS module with various export patterns
const path = require("node:path");

// Regular exports
exports.join = function (a, b) {
	return path.join(a, b);
};

exports.basename = function (filepath) {
	return path.basename(filepath);
};

exports.dirname = function (filepath) {
	return path.dirname(filepath);
};

// Properties
exports.sep = path.sep;
exports.delimiter = path.delimiter;

// Object export
exports.utils = {
	normalize: function (filepath) {
		return path.normalize(filepath);
	},
	resolve: function (...segments) {
		return path.resolve(...segments);
	},
	relative: function (from, to) {
		return path.relative(from, to);
	}
};

// Function that returns object
exports.createPathHandler = function (basePath) {
	return {
		resolve: file => path.resolve(basePath, file),
		relative: file => path.relative(basePath, file)
	};
};

// Unused exports for testing
exports.unusedFunction = function () {
	return "this should show as unused";
};

exports.anotherUnusedExport = {
	property: "unused"
};
