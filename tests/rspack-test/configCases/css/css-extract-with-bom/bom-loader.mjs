// Mimics loaders that prepend a UTF-8 BOM to their output, e.g. dart-sass.
export default function (source) {
	return "\uFEFF" + source;
};
