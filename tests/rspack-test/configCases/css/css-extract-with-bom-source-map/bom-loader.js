// Mimics loaders that prepend a UTF-8 BOM to their output, e.g. dart-sass.
module.exports = function (source) {
	return "\uFEFF" + source;
};
